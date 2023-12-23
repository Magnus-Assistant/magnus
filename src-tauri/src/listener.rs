use std::env;
use serde_json;
use reqwest::Error;
use crate::globals::get_reqwest_client;

pub async fn initialize() -> Result<(), Error> {
    // should init only one client and keep it alive
    let client = get_reqwest_client();

    let openai_key = env::var("OPENAI_KEY").unwrap_or_else(|_| {
        panic!("Unable to fetch OpenAI key")
    });

    let map = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [{"role": "user", "content": "Say this is a test!"}],
        "temperature": 0.7,
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", openai_key))
        .json(&map)
        .send()
        .await?;

    let status = response.status();

    let body = response.json::<serde_json::Value>().await?;

    if status.is_success() && body["choices"][0]["message"]["content"] == "This is a test!" {
        println!("Listener initialization successful.");
    }
    else {
        println!("Listener initialization FAILED.");
    }

    Ok(())
}
