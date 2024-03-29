use lazy_static::lazy_static;
use std::path::PathBuf;
use Permission::*;
use serde_json::{json, to_string_pretty, Value};
use std::{fs::File, io::{Read, Write}};
use tauri::api::path::local_data_dir;

pub enum Permission {
    Clipboard,
    Location,
    Microphone,
    Screenshot,
    Tts
}

impl Permission {
    pub fn as_str(&self) -> &str {
        match *self {
            Clipboard => "Clipboard",
            Location => "Location",
            Microphone => "Microphone",
            Screenshot => "Screenshot",
            Tts => "Tts"
        }
    }
}

lazy_static! {
    static ref PERMISSIONS_FILE: PathBuf = {
        let mut path = local_data_dir().unwrap();
        path.push("magnus");
        path.push("permissions.json");
        path
    };
}

pub fn update(permissions: Value) {
    let permissions = json!({
        "Clipboard": true,
        "Location": true,
        "Microphone": true,
        "Screenshot": true,
        "Tts": true
    });
    let pretty_json = to_string_pretty(&permissions).unwrap();
    let mut file = File::create(PERMISSIONS_FILE.to_str().unwrap()).expect("Failed to create permissions.json!");
    file.write_all(pretty_json.as_bytes()).expect("Failed to write to file");
}

pub fn get() -> Value {
    let mut file = File::open(PERMISSIONS_FILE.to_str().unwrap()).expect("Failed to open permissions.json!");
    let mut json_string = String::new();
    file.read_to_string(&mut json_string).expect("Failed to read permissions.json!");
    let permissions: Value = serde_json::from_str(&json_string).expect("Failed to parse permissions.json!");
    permissions
}
