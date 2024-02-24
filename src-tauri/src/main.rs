// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crossbeam::channel::{bounded, Receiver, Sender};
use tauri::GlobalShortcutManager;
use std::thread;

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
async fn create_message(message: String) {
    let data = serde_json::json!({
        "role": "user",
        "content": message
    });

    // add message to the thread of messages
    let _ = assistant::create_message(data, globals::get_thread_id()).await;

    // create a run id
    let run_id: String = assistant::create_run(globals::get_thread_id())
        .await
        .unwrap_or_else(|err| {
            panic!("Error occurred: {:?}", err);
        });

    // run the thread and wait for it to finish
    let _ = assistant::run_and_wait(&run_id, globals::get_thread_id()).await;

    // get response from the assistant
    let response = assistant::get_assistant_last_response(globals::get_thread_id()).await.unwrap();

    // speak
    // let _ = tts_utils::speak(response, ).await;
}

fn main() {
    dotenv::dotenv().ok();
        
    /* 
    let (transcription_sender, transcription_receiver): (Sender<String>, Receiver<String>) = bounded::<String>(1);

    // audio input
    thread::spawn(move || {
        audio_input::run(transcription_sender.clone());
    });

    // assistant and audio output
    thread::spawn(move || {
        // create a message thread first
        tauri::async_runtime::block_on(async {
            create_message_thread().await;
        });
        audio_output::run(transcription_receiver);
    });
    */

    // loads the vosk model before the app builds
    let _ = globals::get_vosk_model();
    
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            let mut shortcuts = app_handle.global_shortcut_manager();
            let _ = shortcuts.register("Alt+M", || {
                thread::spawn(move || {
                    let transcription = audio_input::run();

                    match transcription {
                        Some(transcription) => println!("GOT T: {transcription}"),
                        None => println!("NO T")
                    }
                });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
