// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assistant;
mod tools;
mod globals;

use dotenv;

#[tauri::command]
async fn print_messages() -> Result<(), String> {
  let result = assistant::print_messages(globals::get_thread_id()).await;

  match result {
    Ok(_) => Ok(()),
    Err(e) => Err(format!("Error printing messages: {:?}", e))
  }
}

#[tauri::command]
async fn create_message_thread() -> Result<(), String> {
  let result = assistant::create_message_thread().await;

  match result {
    Ok(thread_id) => {
      globals::set_thread_id(thread_id.clone().trim_matches('\"').to_string());
      println!("thread: {}\n---------------------------------------", globals::get_thread_id());
      Ok(())
    },
    Err(e) => {
      Err(format!("Error creating thread_id: {}", e))
    }
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
  let run_id: String = assistant::create_run(globals::get_thread_id()).await.unwrap_or_else(|err| {
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

  tauri::Builder::default()
  .invoke_handler(tauri::generate_handler![create_message_thread, create_message, print_messages])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  
  // initialize model
  // start audio stream (sending to model, needs to wait for the model to initialize)
}
