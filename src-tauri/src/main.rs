// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cpal::traits::DeviceTrait;
use crossbeam::channel::{bounded, Receiver, Sender};
use serde_json::Value;
use std::thread;

mod assistant;
mod audio_input;
mod globals;
mod tools;
mod transcription;
mod tts_utils;

async fn create_message_thread() -> String {
    let result = assistant::create_message_thread().await;

    match result {
        Ok(thread_id) => {
            globals::set_thread_id(thread_id.clone().trim_matches('\"').to_string());
            thread_id
        }
        Err(_) => panic!("Error creating the message thread!"),
    }
}

#[tauri::command]
async fn create_message(message: String, has_tts: bool) -> String {
    let data: Value = serde_json::json!({
        "role": "user",
        "content": message
    });
    let cloned_user_data = data.clone();

    // add message to the thread of messages
    match assistant::create_message(cloned_user_data, globals::get_thread_id()).await {
        Ok(_) => {}
        Err(e) => {
            println!("ERROR in create message {}", e)
        }
    }

    // create a run id
    let run_id: String = assistant::create_run(globals::get_thread_id())
        .await
        .unwrap_or_else(|err| {
            panic!("Error occurred: {:?}", err);
        });

    // run the thread and wait for it to finish
    let _ = assistant::run_and_wait(&run_id, globals::get_thread_id()).await;

    // get response from the assistant
    let response = assistant::get_assistant_last_response(globals::get_thread_id())
        .await
        .unwrap();

    // speak
    println!("Has TTS: {}", has_tts);
    if has_tts {
        tts_utils::speak(response.clone());
    }
    response.trim_matches('"').to_string()
}

fn main() {
    dotenv::dotenv().ok();

    let (a_sender, audio_receiver): (Sender<Vec<i16>>, Receiver<Vec<i16>>) = bounded::<Vec<i16>>(1);
    let (t_sender, transcription_receiver): (Sender<String>, Receiver<String>) =
        bounded::<String>(1);
    let default_input_device = audio_input::get_audio_input_device();
    let audio_config = default_input_device.default_input_config().unwrap();

    //audio input
    let audio_sender = a_sender.clone();
    thread::spawn(move || {
        audio_input::run(audio_sender);
    });

    // transcription
    let transcription_sender = t_sender.clone();
    thread::spawn(move || {
        transcription::run(
            audio_receiver,
            transcription_sender,
            audio_config.sample_rate(),
        );
    });

    // assistant
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(async {
        create_message_thread().await;
        assistant::run(transcription_receiver).await;
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![create_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
