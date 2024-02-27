// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crossbeam::channel::{bounded, Receiver, Sender};
use lazy_static::lazy_static;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{async_runtime, App, AppHandle, GlobalShortcutManager, Manager};
use tokio::runtime::Runtime;

mod assistant;
mod audio_input;
mod audio_output;
mod globals;
mod tools;

lazy_static! {
    static ref TRANSCRIPTION_CHANNEL: (Sender<String>, Receiver<String>) = bounded::<String>(1);
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
async fn run_conversation_flow(app_handle: AppHandle, user_message: Option<String>) -> Option<String> {
    // if we have no user message, attempt to get speech input
    let user_message = match user_message {
        Some(message) => Some(message),
        None => audio_input::run(),
    };

    // if there is a user message from either text or speech input, run the flow
    match user_message {
        Some(user_message) => {
            println!("User: {user_message}");
            let _ = app_handle.emit_all("message", Payload { message: user_message.clone() });
            let assistant_message = assistant::run(user_message).await;
            println!("Magnus: {assistant_message}");
            let _ = app_handle.emit_all("message", Payload { message: assistant_message.clone() });

            let assistant_message_clone = assistant_message.clone();
            thread::spawn(move || {
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    let _ = audio_output::speak(assistant_message_clone).await;
                });
            });
            return Some(assistant_message.replace('"', ""));
        }
        None => {
            return None;
        }
    }
}

fn main() {
    dotenv::dotenv().ok();

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
        .invoke_handler(tauri::generate_handler![run_conversation_flow])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
