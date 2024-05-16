use std::env;

use crate::globals::get_reqwest_client;
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
                    "http://localhost:3000".to_string()
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
    let url: String = format!("{}/api/user", get_domain());

    println!("{}", url);

    let user = serde_json::json!({
        "userId": user.user_id,
        "username": user.username,
        "email": user.email,
        "created_at": user.created_at
    });

    // send request to create user.
    // we are doing all input validation on the backend
    let response = get_reqwest_client()
        .post(url)
        .header("Content-Type", "application/json")
        .json(&user)
        .send()
        .await?;

    println!("Creating user status: {:?}", response.text().await);

    Ok(())
}
