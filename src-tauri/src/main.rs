// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use vosk::{Model, Recognizer};

use audio_stream::InputClip;
use crossbeam::channel::{unbounded, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;
mod audio_stream;

mod assistant;
mod globals;
mod tools;

use dotenv;

struct AppState {
    stream_sender: Option<Sender<()>>,
}

/// Converts the F32 (floating point) values to the needed 16bit PCM type for processing
fn convert_to_16pcm(clip_data: &Vec<f32>) -> Vec<i16> {
    let pcm_samples: Vec<i16> = clip_data
        .iter()
        .map(|&sample| {
            // Scale and convert to 16-bit PCM
            (sample * i16::MAX as f32).clamp(-i16::MAX as f32, i16::MAX as f32) as i16
        })
        .collect();

    pcm_samples
}

fn start_model(data_stream: &Vec<i16>) {
    println!("Starting Vosk model with live audio...");
    //grab the stream data so we can dynamically read audio based on what the
    //system assigns for the config
    let stream_data = InputClip::build_config();
    let start = SystemTime::now();

    let model_path = "./models/vosk-model-en-us-0.42-gigaspeech/";

    let model = Model::new(model_path).unwrap();
    let mut recognizer =
        Recognizer::new(&model, stream_data.config.sample_rate().0 as f32).unwrap();

    recognizer.set_max_alternatives(10);
    recognizer.set_words(true);
    recognizer.set_partial_words(true);

    let stop = SystemTime::now();
    match stop.duration_since(start) {
        Ok(t) => println!("Finished Loading model... Took => {:?}", t),
        Err(t) => println!("Error getting time: {}", t),
    };

    println!("Processing Audio Data...");
    let start = SystemTime::now();
    // prints out the partial results. Often times this prints a LOT
    for sample in data_stream.chunks(100) {
        recognizer.accept_waveform(sample);
    }

    println!("{:#?}", recognizer.final_result().multiple().unwrap());
    let stop = SystemTime::now();

    match stop.duration_since(start) {
        Ok(t) => println!("Finished Processing... Took => {:?}", t),
        Err(t) => println!("Error getting time: {}", t),
    };
}

#[tauri::command]
///Starts an audio input stream
fn start_stream(state: tauri::State<Arc<Mutex<AppState>>>) {
    //create the sender so we can add it to state and the receiver for the thread
    let (stream_sender, stream_receiver) = unbounded::<()>();
    state.lock().unwrap().stream_sender = Some(stream_sender);

    //clone it because we are passing ownership to the thread
    let receiver = stream_receiver.clone();

    //spawn a thread that hold the ongoing input stream
    thread::spawn(move || {
        let handle = InputClip::create_stream();
        match receiver.recv() {
            Ok(_) => {
                println!("Stopping stream...");
            }
            Err(e) => eprintln!("Error receiving signal: {}", e),
        }

        //after the stop is received we want to drop the stream object and return the InputClip that was made
        let clip = handle.stop();
        let transformed = InputClip::resample_clip(clip);

        //once we have the needed InputClip we start the model on that audio
        start_model(&convert_to_16pcm(&transformed.samples));
    });
}

#[tauri::command]
///Stops an audio input stream by sending a stop signal
fn stop_stream(state: tauri::State<Arc<Mutex<AppState>>>) {
    // try to obtain a sender so we can use it
    if let Some(sender) = &state.lock().unwrap().stream_sender {
        let sender_clone = sender.clone();
        if sender_clone.send(()).is_err() {
            println!("Failed to send stop signal to stream thread.");
        }
    }
}

#[tauri::command]
async fn print_messages() -> Result<(), String> {
    let result = assistant::print_messages(globals::get_thread_id()).await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error printing messages: {:?}", e)),
    }
}

#[tauri::command]
async fn create_message_thread() -> Result<(), String> {
    let result = assistant::create_message_thread().await;

    match result {
        Ok(thread_id) => {
            globals::set_thread_id(thread_id.clone().trim_matches('\"').to_string());
            println!(
                "thread: {}\n---------------------------------------",
                globals::get_thread_id()
            );
            Ok(())
        }
        Err(e) => Err(format!("Error creating thread_id: {}", e)),
    }
}

#[tauri::command]
async fn create_message(message: String) -> Result<(), String> {
    let data = serde_json::json!({
      "role": "user",
      "content": message
    });
    // add message to the thread of messages
    let _ = assistant::create_message(data, globals::get_thread_id()).await;
    println!("message: {}", message);

    // create a run id
    let run_id: String = assistant::create_run(globals::get_thread_id())
        .await
        .unwrap_or_else(|err| {
            panic!("Error occurred: {:?}", err);
        });
    // println!("run: {}", run_id);

    // run the thread and wait for it to finish
    let _ = assistant::run_and_wait(&run_id, globals::get_thread_id()).await;

    // lets see the response from the assistant
    let _ = assistant::print_assistant_last_response(globals::get_thread_id()).await;

    Ok(())
}

fn main() {
    // loads evironment variables
    dotenv::dotenv().ok();

    //initialize app state
    let app_state = Arc::new(Mutex::new(AppState {
        stream_sender: None,
    }));

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_stream,
            stop_stream,
            create_message_thread,
            create_message,
            print_messages
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
