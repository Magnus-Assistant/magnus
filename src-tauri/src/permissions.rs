use Permission::*;
use serde_json::{to_string_pretty, Value};
use std::{fs::{self, File}, io::Read};

#[derive(Clone)]
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

pub fn update(permissions: Value) {
    let pretty_json = to_string_pretty(&permissions).unwrap();
    let _ = fs::write("permissions.json", pretty_json.as_bytes()).expect("Failed to update permissions.json!");
}

fn get() -> Value {
    let mut file = File::open("permissions.json").expect("Failed to open permissions.json!");
    let mut json_string = String::new();
    file.read_to_string(&mut json_string).expect("Failed to read permissions.json!");
    let permissions: Value = serde_json::from_str(&json_string).expect("Failed to parse permissions.json!");
    permissions
}

pub fn check(required: Vec<Permission>) -> Option<String> {
    let permissions = get();
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
