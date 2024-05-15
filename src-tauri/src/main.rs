// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cpal::traits::DeviceTrait;
use db::{add_user, User};
use dotenv;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, GlobalShortcutManager, Manager};
use tokio::runtime::Runtime;

mod assistant;
mod audio_input;
mod audio_output;
mod db;
mod globals;
mod settings;
mod tools;

lazy_static! {
    static ref APP_HANDLE: Arc<Mutex<Option<AppHandle>>> = Arc::new(Mutex::new(None));
}

#[derive(Clone, serde::Serialize)]
pub struct Payload {
    message: String,
}

async fn create_message_thread() -> String {
    let result = assistant::create_message_thread().await;

    match result {
        Ok(thread_id) => {
            globals::set_thread_id(thread_id.clone().trim_matches('\"').to_string());
            println!("Successfully created thread: {}", globals::get_thread_id());
            thread_id
        }
        Err(_) => panic!("Error creating the message thread!"),
    }
}

#[tauri::command]
async fn create_user(user: User) {
    match add_user(user).await {
        Ok(_) => println!("Created user"),
        Err(err) => {
            println!("Error creating user: {}", err)
        }
    }
}

#[tauri::command]
fn set_is_signed_in(is_signed_in: bool) {
    globals::set_is_signed_in(is_signed_in)
}

#[tauri::command]
fn get_auth_client_id() -> String {
    globals::get_auth_client_id().to_string()
}

#[tauri::command]
fn get_auth_domain() -> String {
    globals::get_auth_domain().to_string()
}

#[tauri::command]
fn get_permissions() -> Value {
    settings::get_permissions()
}

#[tauri::command]
fn update_permissions(permissions: Value) {
    settings::update_permissions(permissions)
}

#[tauri::command]
fn get_audio_input_devices() -> Value {
    let input_devices = audio_input::get_audio_input_device_list()
        .iter()
        .map(|device| Into::<Value>::into(device.name().unwrap()))
        .collect::<Vec<Value>>();
    let current_device = audio_input::get_current_audio_input_device();

    Value::Object(
        json!({
            "devices": input_devices,
            "selected": current_device.name().unwrap()
        })
        .as_object()
        .unwrap()
        .clone(),
    )
}

#[tauri::command]
fn get_audio_output_devices() -> Value {
    let output_devices = audio_output::get_audio_output_device_list()
        .iter()
        .map(|device| Into::<Value>::into(device.name().unwrap()))
        .collect::<Vec<Value>>();
    let current_device = audio_output::get_current_audio_output_device();

    Value::Object(
        json!({
            "devices": output_devices,
            "selected": current_device.name().unwrap()
        })
        .as_object()
        .unwrap()
        .clone(),
    )
}

#[tauri::command]
fn audio_input_device_selection(device_name: String) {
    let mut settings = settings::get_settings().as_object_mut().unwrap().clone();
    settings.insert(
        "audioInputDeviceSelection".to_string(),
        Into::<Value>::into(device_name),
    );
    settings::update_settings(Into::<Value>::into(settings));
}

#[tauri::command]
fn audio_output_device_selection(device_name: String) {
    let mut settings = settings::get_settings().as_object_mut().unwrap().clone();
    settings.insert(
        "audioOutputDeviceSelection".to_string(),
        Into::<Value>::into(device_name),
    );
    settings::update_settings(Into::<Value>::into(settings));
}

#[tauri::command]
async fn run_conversation_flow(app_handle: AppHandle, user_message: Option<String>) {
    // if we have no user message, attempt to get speech input
    let user_message = match user_message {
        Some(message) => Some(message),
        None => audio_input::run(),
    };

    // if there is a user message from either text or speech input, run the flow
    match user_message {
        Some(user_message) => {
            println!("User: {user_message}");
            let _ = app_handle.emit_all(
                "user",
                Payload {
                    message: user_message.clone(),
                },
            );

            let assistant_message = assistant::run(user_message).await;
            println!("Magnus: {assistant_message}");
            let _ = app_handle.emit_all(
                "magnus",
                Payload {
                    message: assistant_message.clone(),
                },
            );

            // exclude code snippets from tts
            let code_snippets_regex = Regex::new(r"`{3}[\s\S]+?`{3}").unwrap();
            let text_to_speak = code_snippets_regex
                .split(&assistant_message)
                .collect::<Vec<_>>()
                .join("\n");
            let should_tts: bool = settings::get_permissions()
                .get("Tts")
                .unwrap()
                .as_bool()
                .unwrap();

            if should_tts && text_to_speak.trim() != "" {
                thread::spawn(move || {
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async {
                        let _ = audio_output::speak(text_to_speak.clone()).await;
                    });
                });
            }
        }
        None => {
            println!("No message from user");
        }
    }
}

fn main() {
    // load env
    if cfg!(debug_assertions) {
        dotenv::dotenv().ok();
        println!("dev!!!!");
    } else {
        #[cfg(target_os = "windows")]
        let env = include_str!("..\\.env");

        #[cfg(target_os = "macos")]
        let env = include_str!("../.env");

        for line in env.lines() {
            // skip empty lines and comments
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                // trim potential whitespace
                let key = key.trim();
                let value = value.trim();

                // set environment variable
                std::env::set_var(key, value);
            }
        }
        println!("prod!!!!");
    }

    // setups before app build
    let running_keybind_flow = Arc::new(Mutex::new(false));

    let _ = globals::get_vosk_model();

    tauri::async_runtime::block_on(async {
        create_message_thread().await;
    });

    tauri::Builder::default()
        .setup(move |app| {
            let app_handle = app.handle();
            *APP_HANDLE.lock().unwrap() = Some(app_handle.clone());
            let mut shortcuts = app_handle.global_shortcut_manager();
            let running_keybind_flow_clone = running_keybind_flow.clone();

            // keybind to begin mic input and assistant response flow, might make this adjustable later
            let _ = shortcuts.register("Alt+M", move || {
                let mut running_keybind_flow = running_keybind_flow_clone.lock().unwrap();

                // limits this to one flow/thread at a time
                if !*running_keybind_flow {
                    *running_keybind_flow = true;
                    let running_keybind_flow_clone = running_keybind_flow_clone.clone();

                    let app_handle_clone = app_handle.clone();
                    // begin flow

                    tauri::async_runtime::spawn(async move {
                        run_conversation_flow(app_handle_clone, None).await;
                        let mut running_keybind_flow = running_keybind_flow_clone.lock().unwrap();
                        *running_keybind_flow = false;
                    });
                } else {
                    println!("Already running keybind flow!");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            run_conversation_flow,
            get_permissions,
            update_permissions,
            get_audio_input_devices,
            get_audio_output_devices,
            audio_input_device_selection,
            audio_output_device_selection,
            get_auth_client_id,
            get_auth_domain,
            set_is_signed_in,
            create_user
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
