// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assistant;
mod tools;
mod globals;

use base64::prelude::{Engine as _, BASE64_STANDARD_NO_PAD};
use scrap::{Capturer, Display};
use image::{Rgba, ImageEncoder, ImageBuffer, ColorType::Rgba8, codecs::png::PngEncoder, imageops::FilterType::Triangle, imageops::resize};
use std::{io::ErrorKind::WouldBlock, time::Duration, thread::sleep, path::Path, fs::File};

#[tauri::command]
fn capture_screen() {
    let display = Display::primary().expect("Couldn't find primary display.");
    let width: u32 = display.width().try_into().unwrap();
    let height: u32 = display.height().try_into().unwrap();
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

    loop {
        // wait for a frame
        let buffer = match capturer.frame() {
            Ok(buffer) => buffer,
            Err(e) if e.kind() == WouldBlock => {
                sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => panic!("Error: {}", e),
        };

        // convert the image data to an image
        let img = ImageBuffer::from_fn(width, height, |x, y| {
            let index = 4 * (y * width + x) as usize;
            let data = &buffer[index..index+4];
            Rgba([data[0], data[1], data[2], data[3]])
        });

        // resize the image -> smallest side (height) must be < 768 px for high res model, and < 512 for low res model
        let new_height: u32 = 768; // or 512
        let new_width: u32 = new_height * width / height;
        let resized_img = resize(&img, new_width, new_height, Triangle);

        // save the image into a new vec
        let mut bytes: Vec<u8> = Vec::new();
        PngEncoder::new(&mut bytes).write_image(&resized_img, new_width, new_height, Rgba8).unwrap();

        // encode the image data to base64
        let base64_image = &BASE64_STANDARD_NO_PAD.encode(&bytes);
        println!("first 10: {:?}\nlast 10: {:?}", base64_image.get(..10), base64_image.get(base64_image.len()-10..));

        // write image for now, for viewing purposes. this will be removed later
        let path = Path::new("C:/Users/schre/Projects/screenshot.png");
        let file = File::create(path).expect("Couldn't create output file.");
        PngEncoder::new(file).write_image(&resized_img, new_width, new_height, Rgba8).expect("Couldn't encode frame.");

        // only need one frame
        break;
    }
}

use std::ptr;
use winapi::um::winuser::{GetClipboardData, OpenClipboard, CloseClipboard, CF_UNICODETEXT};
use winapi::um::winbase::{GlobalLock, GlobalUnlock};
use std::os::windows::ffi::{OsStringExt, OsStrExt};

#[tauri::command]
fn get_clipboard_text() {
  unsafe {
    if OpenClipboard(ptr::null_mut()) == 0 {
        println!("Unable to open cliboard");
    }

    let clipboard_data = GetClipboardData(CF_UNICODETEXT);
    if clipboard_data.is_null() {
        println!("Unable to get cliboard data");
        CloseClipboard();
    }

    let text_ptr = GlobalLock(clipboard_data) as *const u16;
    if text_ptr.is_null() {
        println!("Unable to lock global memory");
        CloseClipboard();
    }

    let text_slice = std::slice::from_raw_parts(text_ptr, {
        let mut len = 0;
        while *text_ptr.offset(len) != 0 {
            len += 1;
        }
        len as usize
    });

    let selected_text = String::from_utf16_lossy(&std::ffi::OsString::from_wide(text_slice).encode_wide().collect::<Vec<_>>());

    GlobalUnlock(clipboard_data);
    CloseClipboard();

    println!("{}", selected_text);
  }
}

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
        .invoke_handler(tauri::generate_handler![create_message_thread, create_message, print_messages, get_clipboard_text, capture_screen])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
  
    // initialize model
    // start audio stream (sending to model, needs to wait for the model to initialize)
}
