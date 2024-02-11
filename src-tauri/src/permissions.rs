use lazy_static::lazy_static;
use std::collections::HashMap;
use Permission::*;
use std::fs::File;
use std::io::{Read, Write};
use serde_json::{json, to_string_pretty, Value};

// currently this is stored in a permissions.json file in the src-tauri dir
// I believe we will need to either do sql or some other option
// because the json file might not work in production builds

enum Permission {
    UserLocation,
    Clipboard,
    ScreenCapture
}

impl Permission {
    // internal string representation
    fn as_str(&self) -> &str {
        match *self {
            UserLocation => "UserLocation",
            Clipboard => "Clipboard",
            ScreenCapture => "ScreenCapture",
        }
    }

    // user and model facing string representation
    fn value(&self) -> &str {
        match *self {
            UserLocation => "Location",
            Clipboard => "Clipboard",
            ScreenCapture => "Screen Capture",
        }
    }
}

lazy_static! {
    static ref PERMISSIONS_REQUIRED: HashMap<&'static str, Vec<Permission>> = {
        let mut map = HashMap::new();

        // create entry for every tool call that needs any permissions
        map.insert("get_user_coordinates", vec![UserLocation]);
        map.insert("get_clipboard_text", vec![Clipboard]);
        map.insert("get_screenshot", vec![ScreenCapture]);

        map
    };
}

// this will eventually be called with values from the settings page on "save changes"
pub fn update() {
    let permissions = json!({
        "UserLocation": true,
        "Clipboard": false,
        "ScreenCapture": false
    });

    let pretty_json = to_string_pretty(&permissions).unwrap();

    let mut file = File::create("permissions.json").expect("Failed to create permissions.json!");
    file.write_all(pretty_json.as_bytes()).expect("Failed to write to file");
}

pub fn get() -> Value {
    let mut file = File::open("permissions.json").expect("Failed to open permissions.json!");
    let mut json_string = String::new();
    file.read_to_string(&mut json_string).expect("Failed to read permissions.json!");

    let permissions: Value = serde_json::from_str(&json_string).expect("Failed to parse permissions.json!");
    permissions
}

pub fn check(tool_call: &str) -> Option<String> {
    if !PERMISSIONS_REQUIRED.contains_key(tool_call) { return None };

    let we_have_permission_to = get();
    let mut need_permission_to: Vec<&str> = vec![];

    // for all permissions required for this tool call, check if the user has given permission
    for permission in &PERMISSIONS_REQUIRED[tool_call] {
        if !we_have_permission_to[permission.as_str()].as_bool().unwrap() {
            need_permission_to.push(permission.value());
        }
    }

    if need_permission_to.len() > 0 {
        Some(format!("You MUST tell the user they need to allow access to ALL of the following features in settings: {}", need_permission_to.join(", ")))
    }
    else {
        None
    }
}