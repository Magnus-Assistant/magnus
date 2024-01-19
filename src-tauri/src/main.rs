// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use audio_stream::InputClip;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod assistant;
mod audio_stream;
mod globals;
mod model_utils;
mod tools;
mod tts_utils;

struct AppState {
    stream_sender: Option<Sender<String>>,
    result_sender: Option<Sender<AudioStreamResult>>,
    result_receiver: Option<Receiver<AudioStreamResult>>,
}

#[derive(Debug)]
enum AudioStreamResult {
    Result(String),
}

mod audio_input;
mod transcription;

#[tauri::command]
///Starts an audio input stream and sends the results to a language model
fn start_stream(state: tauri::State<Arc<Mutex<AppState>>>) {
    //create the sender and recievers so we can add it to state
    let (stream_sender, stream_receiver) = unbounded::<String>();
    let stream_receiver_clone = stream_receiver.clone();
    let (result_sender, result_reciever) = unbounded::<AudioStreamResult>();
    let result_sender_clone = result_sender.clone();

    //add senders and receivers to tauri state
    state.lock().unwrap().stream_sender = Some(stream_sender);
    state.lock().unwrap().result_sender = Some(result_sender);
    state.lock().unwrap().result_receiver = Some(result_reciever);

    //spawn a thread that holds the ongoing input stream
    thread::spawn(move || {
        let handle = InputClip::create_stream();

        //wait for the kill signal to stop the stream
        match stream_receiver_clone.recv() {
            Ok(_) => {
                println!("Stopping stream...");
            }
            Err(e) => eprintln!("Error receiving signal to stop stream: {}", e),
        }

        //after the kill is received we want to drop the stream object and return the InputClip that was made
        let clip = handle.stop();
        let transformed = InputClip::resample_clip(clip);

        //once we have the needed InputClip we start the model on that audio and send its results to the main thread
        let model_results =
            model_utils::start_model(&audio_stream::convert_to_16pcm(&transformed.samples));

        let final_result = AudioStreamResult::Result(model_results);
        result_sender_clone.send(final_result).unwrap();
    });
}


// start stream (on app start)
// loop:
// wait 200ms
// samples = get audio samples (clear buffer)
// if recognizer.accept_waveform(samples) == Finalized -> get stream results -> message to assisstant

#[tauri::command]
///returns the string output of the vosk model
fn get_stream_results(state: tauri::State<Arc<Mutex<AppState>>>) -> Option<String> {
    if let Some(receiver) = &state.lock().unwrap().result_receiver {
        /*
        If we request for the results and its not finished or doesn't exist we dont want to
        block the thread forever so we timeout on the recv.
        */
        match receiver.recv_timeout(Duration::from_millis(200)) {
            Ok(status) => match status {
                //if we successfully received a result return it
                AudioStreamResult::Result(output) => Some(output),
            },
            Err(_) => {
                println!("Get stream results request timed out... Is the stream completed?");
                None
            }
        }
    } else {
        println!("Failed to get Receiver for stream results");
        None
    }
}

#[tauri::command]
///Stops an audio input stream by sending a stop signal
fn stop_stream(state: tauri::State<Arc<Mutex<AppState>>>) {
    // try to obtain a sender so we can use it to stop the stream
    if let Some(sender) = &state.lock().unwrap().stream_sender {
        let sender_clone = sender.clone();
        if sender_clone.send("stop stream".to_string()).is_err() {
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
async fn create_message(message: String) {
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

    // get response from the assistant
    let response = assistant::get_assistant_last_response(globals::get_thread_id()).await.unwrap();

    // print and speak
    println!("response: {}", response.clone());
    tts_utils::speak(response);
}

fn main() {
    // loads evironment variables
    dotenv::dotenv().ok();

    //initialize app state
    let app_state = Arc::new(Mutex::new(AppState {
        stream_sender: None,
        result_sender: None,
        result_receiver: None,
    }));

    let (a_sender, audio_receiver): (Sender<Vec<i16>>, Receiver<Vec<i16>>) = unbounded::<Vec<i16>>();
    let (t_sender, transcription_receiver): (Sender<String>, Receiver<String>) = unbounded::<String>();

    // audio input
    let audio_sender = a_sender.clone();
    thread::spawn(move || {
        audio_input::start_audio_stream(audio_sender);
    });

    // transcription
    let transcription_sender = t_sender.clone();
    thread::spawn(move || {
        transcription::run(audio_receiver, transcription_sender);
    });

    // assistant
    thread::spawn(move || {
        loop {
            if let Ok(data) = transcription_receiver.recv() {
                println!("{data:?}");
            }
        }
    });
    
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            create_message_thread,
            create_message,
            get_stream_results,
            start_stream,
            stop_stream,
            print_messages
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
