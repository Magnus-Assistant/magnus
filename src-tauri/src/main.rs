// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assistant;
mod tools;
mod globals;

#[tauri::command]
async fn print_messages() {
    let _ = assistant::print_messages(globals::get_thread_id()).await;
}

#[tauri::command]
async fn create_message_thread() {
    let result = assistant::create_message_thread().await;

    match result {
    Ok(thread_id) => {
        globals::set_thread_id(thread_id.clone().trim_matches('\"').to_string());
        println!("thread: {}\n---------------------------------------", globals::get_thread_id());
    },
    Err(e) => println!("Error creating thread_id: {}", e),
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
