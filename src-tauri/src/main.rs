// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use vosk::{ Model, Recognizer };

use std::fs::File;
use std::io::Read;
use std::time::SystemTime;
use audio_stream::InputClip;
mod audio_stream;

#[tauri::command]
async fn my_custom_command() {
  println!("{}", "I was invoked from JS!");
}

#[tauri::command]
async fn start_model(data_stream: Vec<i16>) {
  println!("Starting Vosk model with live audio...");
  //grab the stream data so we can dynamically read audio based on what the
  //system assigns for the config
  let stream_data = InputClip::build_config();
  let start = SystemTime::now();

  let test_audio_path: &str = "./test_audio/magnus.wav";

  let mut test_wav_data = Vec::new();
  File::open(test_audio_path)
    .expect("Failed to open")
    .read_to_end(&mut test_wav_data)
    .expect("Failed to read");

  let model_path = "./models/vosk-model-en-us-0.42-gigaspeech/";

  let model = Model::new(model_path).unwrap();
  let mut recognizer = Recognizer::new(&model, stream_data.config.sample_rate().0 as f32).unwrap();

  recognizer.set_max_alternatives(10);
  recognizer.set_words(true);
  recognizer.set_partial_words(true);
  let stop = SystemTime::now();
  match stop.duration_since(start) {
    Ok(t) => println!("Finished Loading model... Took => {:?}", t),
    Err(t) => println!("Error getting time: {}", t)
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
    Err(t) => println!("Error getting time: {}", t)
  };
}

#[tauri::command]
async fn start_test_stream(){

  let stream_clip = InputClip::create_stream().await;
  start_model(stream_clip).await;
}


fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![my_custom_command, start_model, start_test_stream])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
