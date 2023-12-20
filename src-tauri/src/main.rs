// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use vosk::{ Model, Recognizer };

#[tauri::command]
async fn my_custom_command() {
  println!("{}", "I was invoked from JS!");
}

#[tauri::command]
async fn start_model() {
  println!("{}", "Starting the model... maybe");
  // Normally you would not want to hardcode the audio samples
  let samples = vec![100, -2, 700, 30, 4, 5];
  let model_path = "./models/vosk-model-en-us-0.42-gigaspeech/";

  let model = Model::new(model_path).unwrap();
  let mut recognizer = Recognizer::new(&model, 16000.0).unwrap();

  recognizer.set_max_alternatives(10);
  recognizer.set_words(true);
  recognizer.set_partial_words(true);

  for sample in samples.chunks(100) {
      recognizer.accept_waveform(sample);
      println!("{:#?}", recognizer.partial_result());
  }

  println!("{:#?}", recognizer.final_result().multiple().unwrap());
}

  
fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![my_custom_command, start_model])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
