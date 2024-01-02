use crate::globals::{ get_reqwest_client, get_ip_api_key };
use reqwest::Error;

pub fn get_location_weather(location: &str) -> String {
    println!("getting weather of {}!", location);
    format!("Its currently 40 F in {}, with a 45% chance of rain after 8pm.", location)
}

pub fn get_local_weather(latitude: &str, longitude: &str) -> String {
    println!("getting local weather!");
    format!("Weather of {}, {} is warm and sunny!", latitude, longitude)
}

pub async fn get_user_location() -> Result<String, Error> {
    println!("getting user location!");
    let response = get_reqwest_client()
        .get(format!("https://ipapi.co/json/?key={}", get_ip_api_key()))
        .send()
        .await?;
 
    let location = response.json::<serde_json::Value>().await?;
 
    Ok(format!("latitude: {}, longitude: {}", location["latitude"], location["longitude"]))
}

pub fn pass() -> String {
    println!("passing!");
    format!("")
}