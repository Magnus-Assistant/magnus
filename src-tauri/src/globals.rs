use lazy_static::lazy_static;
use reqwest::Client;
use std::env;
use std::sync::Mutex;
use vosk::Model;

lazy_static! {
    static ref REQWEST_CLIENT: Client = Client::new();
    static ref THREAD_ID: Mutex<String> = Mutex::new("".to_string());
    static ref VOSK_MODEL: Model = {
        // let model_path = "./models/vosk-model-en-us-0.42-gigaspeech/";

    // TODO Rework all of this code (lines 13 - 34)
    // its really icky and not fault tolerant
    #[cfg(target_os = "windows")]
    {
        let model_path = "./models/vosk-model-small-en-us-0.15";
        Model::new(model_path).unwrap()
    }

    #[cfg(target_os = "macos")]
    {
        if cfg!(debug_assertions) {
            let model_path = "./models/vosk-model-small-en-us-0.15";
            Model::new(model_path).unwrap()
        } else {
        let mut exe_path = env::current_exe().ok().unwrap();
        exe_path.pop();
        exe_path.pop();
        exe_path.push("Resources");

        let path_str = exe_path.to_str().unwrap();
        let complete_path = format!("{path_str}/models/vosk-model-small-en-us-0.15");
        Model::new(complete_path).unwrap()
        }
    }
    };

    static ref MAGNUS_ID: String = {
        match env::var("MAGNUS_ID") {
            Ok(value) => value,
            Err(_) => panic!("Could not fetch Magnus ID!")
        }
    };
    static ref OPENAI_KEY: String = {
        match env::var("OPENAI_KEY") {
            Ok(value) => value,
            Err(_) => { println!("Could not fetch OpenAI API key!"); return "".to_string() }
        }
    };
    static ref IPAPI_KEY: String = {
        match env::var("IPAPI_KEY") {
            Ok(value) => value,
            Err(_) => { println!("Could not fetch IP API key!"); return "".to_string() }
        }
    };
    static ref WEATHER_API_USER_AGENT: String = {
        match env::var("WEATHER_API_USER_AGENT") {
            Ok(value) => value,
            Err(_) => { println!("Could not fetch weather API User-Agent!"); return "".to_string() }
        }
    };
    static ref OPENCAGE_KEY: String = {
        match env::var("OPENCAGE_KEY") {
            Ok(value) => value,
            Err(_) => { println!("Could not fetch OpenCage API key!"); return "".to_string() }
        }
    };

    static ref AUTH_DOMAIN: String = {
        match env::var("AUTH_DOMAIN") {
            Ok(value) => value,
            Err(_) => { println!("Could not fetch auth domain!"); return "".to_string() }
        }
    };

    static ref AUTH_CLIENT_ID: String = {
        match env::var("AUTH_CLIENT_ID") {
            Ok(value) => value,
            Err(_) => { println!("Could not fetch auth client ID!"); return "".to_string() }
        }
    };
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

pub fn get_opencage_key() -> &'static String {
    &OPENCAGE_KEY
}

pub fn get_weather_api_user_agent() -> &'static String {
    &WEATHER_API_USER_AGENT
}

pub fn get_vosk_model() -> &'static Model {
    &VOSK_MODEL
}

pub fn get_thread_id() -> String {
    THREAD_ID.lock().unwrap().clone()
}

pub fn set_thread_id(new_value: String) {
    *THREAD_ID.lock().unwrap() = new_value;
}

pub fn get_auth_domain() -> &'static String {
    &AUTH_DOMAIN
}

pub fn get_auth_client_id() -> &'static String {
    &AUTH_CLIENT_ID
}
