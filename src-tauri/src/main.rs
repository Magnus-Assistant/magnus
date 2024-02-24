// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::GlobalShortcutManager;
use std::sync::{Arc, Mutex};

mod assistant;
mod globals;
mod tools;
mod audio_output;
mod audio_input;

async fn create_message_thread() -> String {
    let result = assistant::create_message_thread().await;

    match result {
        Ok(thread_id) => {
            globals::set_thread_id(thread_id.clone().trim_matches('\"').to_string());
            println!(
                "Successfully created thread: {}",
                globals::get_thread_id()
            );
            thread_id
        }
        Err(_) => panic!("Error creating the message thread!"),
    }
}

#[tauri::command]
async fn run_conversation_flow(user_message: String) {
    println!("User: {user_message}");
    let assistant_message = assistant::run(user_message).await;
    println!("Magnus: {assistant_message}");

    let _ = audio_output::speak(assistant_message.clone()).await;

    // emit assistant message to frontend
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
                
                    // begin flow
                    tauri::async_runtime::spawn(async move {
                        let transcription = audio_input::run();

                        match transcription {
                            Some(transcription) => run_conversation_flow(transcription).await,
                            None => println!("NONE")
                        }

                        let mut running_keybind_flow = running_keybind_flow_clone.lock().unwrap();
                        *running_keybind_flow = false;
                    });
                }
                else {
                    println!("Already running keybind flow!");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            run_conversation_flow
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
