// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use vosk::{Model, Recognizer};

use audio_stream::InputClip;
use crossbeam::channel::{unbounded, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;
mod audio_stream;

struct AppState {
    stream_sender: Option<Sender<()>>,
}

/// Converts the F32 (floating point) values to the needed 16bit PCM type for processing
fn process_input_data(data: &Vec<f32>) -> Vec<i16> {
    let pcm_samples: Vec<i16> = data
        .iter()
        .map(|&sample| {
            // Scale and convert to 16-bit PCM
            (sample * i16::MAX as f32).clamp(-i16::MAX as f32, i16::MAX as f32) as i16
        })
        .collect();

    pcm_samples
}

#[tauri::command]
async fn my_custom_command() {
    println!("{}", "I was invoked from JS!");
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
fn start_stream(state: tauri::State<Arc<Mutex<AppState>>>) {
    let (stream_sender, stream_receiver) = unbounded::<()>();
    state.lock().unwrap().stream_sender = Some(stream_sender);
    let receiver = stream_receiver.clone();

    thread::spawn(move || {
        let handle = InputClip::create_stream();

        match receiver.recv() {
            Ok(_) => {
                println!("Stopping stream...");
            }
            Err(e) => eprintln!("Error receiving signal: {}", e),
        }

        let clip = handle.stop();
        let transformed = InputClip::resample_clip(clip);
        start_model(&process_input_data(&transformed.samples));

        println!("Recorded {} samples", transformed.samples.len());
    });
}

#[tauri::command]
fn stop_stream(state: tauri::State<Arc<Mutex<AppState>>>) {
    if let Some(sender) = &state.lock().unwrap().stream_sender {
        let sender_clone = sender.clone();
        if sender_clone.send(()).is_err() {
            eprintln!("Failed to send stop signal to stream thread.");
        }
    }
}

fn main() {
    let app_state = Arc::new(Mutex::new(AppState {
        stream_sender: None,
    }));

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            my_custom_command,
            start_stream,
            stop_stream
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
