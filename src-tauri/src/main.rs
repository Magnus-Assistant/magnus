// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, GlobalShortcutManager, Manager};
use tokio::runtime::Runtime;

mod assistant;
mod audio_input;
mod audio_output;
mod globals;
mod permissions;
mod tools; 

lazy_static! {
    static ref HAS_TTS: Mutex<bool> = Mutex::new(false);
}

#[derive(Clone, serde::Serialize)]
struct Payload {
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
async fn set_tts(tts_value: bool) { 
    *HAS_TTS.lock().unwrap() = tts_value;
}

#[tauri::command]
async fn run_conversation_flow(app_handle: AppHandle, user_message: Option<String>) {

    let should_tts = *HAS_TTS.lock().unwrap();
    println!("HAS TTS: {}", should_tts);

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

            if should_tts {
                let assistant_message_clone = assistant_message.clone();
                thread::spawn(move || {
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async {
                        let _ = audio_output::speak(assistant_message_clone).await;
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
    }
    else {
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
        .invoke_handler(tauri::generate_handler![run_conversation_flow, set_tts])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
