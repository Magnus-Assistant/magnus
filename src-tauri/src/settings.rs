use cpal::traits::DeviceTrait;
use Permission::*;
use serde_json::{to_string_pretty, Map, Value};
use std::{ fs::{self, File}, io::Read, path::PathBuf};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;

use crate::{audio_input, audio_output};

pub fn get_magnus_data_dir_path() -> PathBuf {
    let mut path = tauri::api::path::data_dir().unwrap();
    path.push("magnus");
    path    
}

#[derive(Clone, EnumIter)]
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

pub fn get_permissions_file_path() -> PathBuf {
    let mut path = tauri::api::path::data_dir().unwrap();
    path.push("magnus");
    path.push("permissions.json");
    path    
}

pub fn create_permissions() {
    let mut permissions_json = Map::new();
    for permission in Permission::iter() {
        permissions_json.insert(permission.as_str().to_string(), Value::Bool(false));
    }
    let pretty_json = to_string_pretty(&permissions_json).unwrap();

    // create the magnus directory within the system's app data directory
    match fs::create_dir(get_magnus_data_dir_path()) {
        Ok(_) => println!("Created Directory"),
        Err(err) => println!("Error Creating Directory: {}", err),
    }


    // create the permissions.json file with all false values
    let _ = fs::write(get_permissions_file_path(), pretty_json.as_bytes()).expect("Failed to update permissions.json!");
}

pub fn update_permissions(permissions: Value) {
    if permissions != Value::Object(Map::new()) {
        let pretty_json = to_string_pretty(&permissions).unwrap();
        let _ = fs::write(get_permissions_file_path(), pretty_json.as_bytes()).expect("Failed to update permissions.json!");
    }
}

pub fn get_permissions() -> Value {
    match File::open(get_permissions_file_path()) {
        Ok(mut file) => {
            let mut json_string = String::new();
            file.read_to_string(&mut json_string).expect("Failed to read permissions.json!");
            let permissions: Value = serde_json::from_str(&json_string).expect("Failed to parse permissions.json!");

            // fix empty permisisons file
            if permissions == Value::Object(Map::new()) {
                create_permissions();
                return get_permissions()
            }

            return permissions    
        },
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                println!("no permissions.json file!!!");
                create_permissions();
                return get_permissions()
            } else {
                todo!();
            }
        }
    }
}

pub fn check_permissions(required: Vec<Permission>) -> Option<String> {
    let permissions = get_permissions();
    let mut denied: Vec<Permission> = vec![];

    for permission in required {
        let granted = permissions.get(permission.as_str()).unwrap().as_bool().unwrap();

        if !granted {
            denied.push(permission.clone());
            println!("no permission to {}", permission.as_str());
        }
        else {
            println!("permission given for {}", permission.as_str());
        }
    }

    if denied.len() == 0 {
        return None
    }
    else {
        let all_denied: Vec<&str> = denied.iter().map(|p| p.as_str()).collect();

        // this message could potentially use tweaking
        return Some(format!("You MUST tell the user they need to allow access to ALL of the following features in settings: {}", all_denied.join(", ")))
    }
}

pub fn get_settings_file_path() -> PathBuf {
    let mut path = tauri::api::path::data_dir().unwrap();
    path.push("magnus");
    path.push("settings.json");
    path    
}

pub fn create_settings() {
    let settings_json = serde_json::json!({
        "audioInputDeviceSelection": audio_input::get_default_audio_input_device().name().unwrap(),
        "audioOutputDeviceSelection": audio_output::get_default_audio_output_device().name().unwrap()
    }).as_object().unwrap().clone();

    let pretty_json = to_string_pretty(&settings_json).unwrap();

    // create the magnus directory within the system's app data directory
    let _ = fs::create_dir(get_magnus_data_dir_path());

    let _ = fs::write(get_settings_file_path(), pretty_json.as_bytes()).expect("Failed to update settings.json!");
}

pub fn update_settings(settings: Value) {
    if settings != Value::Object(Map::new()) {
        let pretty_json = to_string_pretty(&settings).unwrap();
        let _ = fs::write(get_settings_file_path(), pretty_json.as_bytes()).expect("Failed to update settings.json!");
   }
}

pub fn get_settings() -> Value {
    match File::open(get_settings_file_path()) {
        Ok(mut file) => {
            let mut json_string = String::new();
            file.read_to_string(&mut json_string).expect("Failed to read settings.json!");
            let settings: Value = serde_json::from_str(&json_string).expect("Failed to parse settings.json!");

            // fix empty settings file
            if settings == Value::Object(Map::new()) {
                create_settings();
                return get_settings()
            }

            return settings    
        },
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                println!("no settings.json file!!!");
                create_settings();
                return get_settings()
            }
            else {
                todo!()
            }
        }
    }
}
