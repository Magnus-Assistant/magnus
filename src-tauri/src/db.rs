use std::env;

use crate::globals::{get_auth_jwt, get_auth_user_id, get_reqwest_client};
use lazy_static::lazy_static;

// dynamically choose the domain based on the env value "IS_PROD"
lazy_static! {
    #[derive(Debug)]
    static ref DOMAIN: String = {
        match env::var("IS_PROD") {
            Ok(value) => {
                if value == "true" {
                    "https://magnusbackend.azurewebsites.net".to_string()
                } else {
                    "http://localhost:3000/api".to_string()
                }
            }
            Err(_) => {
                println!("Could not fetch environment status!");
                return "".to_string();
            }
        }
    };
}

pub fn get_domain() -> &'static str {
    &DOMAIN.as_str()
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct User {
    user_id: String,
    username: String,
    email: String,
    created_at: String,
}

pub async fn add_user(user: User) -> Result<(), Box<dyn std::error::Error>> {
    let url: String = format!("{}/user", get_domain());

    let user = serde_json::json!({
        "userId": user.user_id,
        "username": user.username,
        "email": user.email
    });

    // send request to create user.
    // we are doing all input validation on the backend
    let response = get_reqwest_client()
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("{} {}", "Bearer ", get_auth_jwt()))
        .json(&user)
        .send()
        .await?;

    // send request to create user.
    // we are doing all input validation on the backend
    let response = get_reqwest_client()
        .post(url)
        .header("Content-Type", "application/json")
        .json(&user)
        .send()
        .await?;

    Ok(())
}

// Creating and adding logs to our DB
#[derive(serde::Serialize, serde::Deserialize)]
pub enum LogLevels {
    Info = 0,
    Warning = 1,
    Error = 2,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Log {
    pub(crate) user_id: String,
    pub(crate) log_level: LogLevels,
    pub(crate) message: String,
    pub(crate) source: Option<String>,
}

impl Log {
    fn convert_log_level(level: LogLevels) -> i32 {
        match level {
            LogLevels::Info => 0,
            LogLevels::Warning => 1,
            LogLevels::Error => 2,
        }
    }

    pub async fn log(log: Log) -> Result<(), Box<dyn std::error::Error>> {
        let url: String = format!("{}/api/log", get_domain());

        let log = serde_json::json!({
            "userId": log.user_id,
            "logLevel": Self::convert_log_level(log.log_level), // convert here because of serialization weirdness
            "message": log.message,
            "source": log.source,
        });

        // send request to create a log.
        // we are doing all input validation on the backend
        let _ = get_reqwest_client()
            .post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("{} {}", "Bearer ", get_auth_jwt()))
            .json(&log)
            .send()
            .await?;
        Ok(())
    }
}
