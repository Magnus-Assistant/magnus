use reqwest::Client;
use lazy_static::lazy_static;
use std::env;
use std::sync::Mutex;

lazy_static! {
    static ref REQWEST_CLIENT: Client = Client::new();
    static ref MAGNUS_ID: String = {
        match env::var("MAGNUS_ID") {
            Ok(value) => value,
            Err(_) => panic!("Could not fetch Magnus ID!")
        }
    };
    static ref OPENAI_KEY: String = {
        match env::var("OPENAI_KEY") {
            Ok(value) => value,
            Err(_) => panic!("Could not fetch OpenAI key!")
        }
    };
    static ref IPAPI_KEY: String = {
        match env::var("IPAPI_KEY") {
            Ok(value) => value,
            Err(_) => panic!("Could not fetch IP API key!")
        }
    };
    static ref THREAD_ID: Mutex<String> = Mutex::new("".to_string());
}

pub fn get_reqwest_client() -> &'static Client {
    &REQWEST_CLIENT
}

pub fn get_magnus_id() -> &'static String {
    &MAGNUS_ID
}

pub fn get_open_ai_key() -> &'static String {
    &OPENAI_KEY
}

pub fn get_ip_api_key() -> &'static String {
    &IPAPI_KEY
}

pub fn get_thread_id() -> String {
    THREAD_ID.lock().unwrap().clone()
}

pub fn set_thread_id(new_value: String) {
    *THREAD_ID.lock().unwrap() = new_value;
}
