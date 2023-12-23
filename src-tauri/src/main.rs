// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod listener;
mod globals;
use dotenv;

#[tauri::command]
async fn my_custom_command() {
  println!("{}", "I was invoked from JS!");
}


#[tauri::command]
async fn start_listener() -> Result<(), String> {
  let result = listener::initialize().await;
  match result {
    Ok(_) => Ok(()),
    Err(e) => Err(format!("Error performing request: {:?}", e))
  }
}

  
fn main() {
  // loads evironment variables
  dotenv::dotenv().ok();

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![my_custom_command, start_listener])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  
  // initialize model
  // start audio stream (sending to model, needs to wait for the model to initialize)
}
