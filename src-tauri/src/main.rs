// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


use dasp_sample::conv::f32;
use vosk::{ Model, Recognizer };

use std::fs::File;
use std::io::Read;
use std::thread::{self, sleep};
use std::time::Duration;
use audio_stream::AudioClip;

mod audio_stream;

#[tauri::command]
async fn my_custom_command() {
  println!("{}", "I was invoked from JS!");
}

#[tauri::command]
async fn start_model(data_stream: Vec<f32>) {
  println!("{}", "Starting the model... maybe");

  let test_audio_path: &str = "./test_audio/magnus.wav";

  let mut test_wav_data = Vec::new();
  File::open(test_audio_path)
    .expect("Failed to open")
    .read_to_end(&mut test_wav_data)
    .expect("Failed to read");

  let model_path = "./models/vosk-model-en-us-0.42-gigaspeech/";

  let mut vec: Vec<i16> = Vec::new();
  let mut count = 0;
  for i in data_stream.clone() {
    vec.push((i * 10000.0) as i16 );    
    println!("{} ", vec[count]);
    count = count + 1;
  }

  //let samples: Vec<i16> = &data_stream.as_slice().iter().map(|&f| f as i16).collect();
  // for i in samples.clone() {
  //   print!("{} ", i);
  // }
  //let samples: Vec<i16> = data_stream.chunks(2).map(|chunk| i16::from_ne_bytes([chunk[0], chunk[1]])).collect();

  let model = Model::new(model_path).unwrap();
  let mut recognizer = Recognizer::new(&model, 16000.0).unwrap();

  recognizer.set_max_alternatives(10);
  recognizer.set_words(true);
  recognizer.set_partial_words(true);

  for sample in vec.chunks(100) {
      recognizer.accept_waveform(sample);
      println!("{:#?}", recognizer.partial_result());
  }

  println!("{:#?}", recognizer.final_result().multiple().unwrap());
}

#[tauri::command]
async fn start_test_stream(){
  let clip = AudioClip::record("TestAudio".into());
  match clip {
      Ok(c)=> {
        println!("Data length: {}", c.len());
        start_model(c).await
    },
    Err(_) => todo!(), }
}

  
fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![my_custom_command, start_model, start_test_stream])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
