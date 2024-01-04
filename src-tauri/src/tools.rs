use crate::globals::{ get_reqwest_client, get_ip_api_key, get_weather_api_user_agent };
use serde_json::Value;
use chrono::prelude::Local;

pub fn get_location_coordinates(location: &str) -> String {
    println!("getting {} coordinates!", location);
    format!("lat: {}, lng: {}", 40.7128, 74.0060)
}

pub async fn get_forecast(lat: &str, lng: &str, n_days: &str) -> String {
    println!("getting weather!");
    let weather_result = get_reqwest_client()
        .get(format!("https://api.weather.gov/points/{},{}", lat, lng))
        .header("User-Agent", get_weather_api_user_agent())
        .send()
        .await;

    match weather_result {
        Ok(weather_response) => {
            match weather_response.json::<Value>().await {
                Ok(weather) => {
                    let forecast_url = weather["properties"]["forecast"].to_string().trim_matches('"').to_string();
                    let forecast_result = get_reqwest_client()
                        .get(forecast_url)
                        .header("User-Agent", get_weather_api_user_agent())
                        .send()
                        .await;

                    match forecast_result {
                        Ok(forecast_response) => {
                            match forecast_response.json::<Value>().await {
                                Ok(forecast) => {
                                    match forecast["properties"]["periods"].as_array() {
                                        Some(days) => {
                                            let mut the_forecast = "".to_string();
                                            let mut num_days = n_days.parse().unwrap();
                                            num_days *= 2; // because we receive forecast in half days, and assistants gives us n days
                                            for day in 0..num_days {
                                                match days[day].as_object() {
                                                    Some(day_forecast) => {
                                                        println!("{}: {}\n", day_forecast["name"], day_forecast["detailedForecast"]);
                                                        the_forecast.push_str(format!("{}: {}\n", day_forecast["name"], day_forecast["detailedForecast"]).as_str());
                                                    },
                                                    _ => return format!("Couldn't make day into an object.")
                                                }
                                            }
                                            the_forecast
                                        },
                                        _ => return format!("No forecast in response.")
                                    }
                                },
                                Err(e) => format!("Unable to parse forecast response: {}", e)
                            }
                        },
                        Err(e) => format!("Request to get forecast failed: {}", e)
                    }
                },
                Err(e) => format!("Unable to parse weather response: {}", e)
            }
        }
        Err(e) => format!("Request to get weather failed: {}", e)
    }

}

pub async fn get_user_coordinates() -> String {
    println!("getting user's location!");
    let user_coordinates_result = get_reqwest_client()
        .get(format!("https://ipapi.co/json/?key={}", get_ip_api_key()))
        .send()
        .await;

    match user_coordinates_result {
        Ok(user_coordinates_response) => {
            match user_coordinates_response.json::<Value>().await {
                Ok(user_coordinates) => return format!("lat: {}, lng: {}", user_coordinates["latitude"], user_coordinates["longitude"]),
                Err(e) => return format!("Unable to parse response: {}", e)
            }
        }
        Err(e) => format!("Request to get user's coordinates failed: {}", e)
    }
}

pub fn get_time() -> String {
    println!("{:#?}", Local::now());
    format!("{:#?}", Local::now())
}

pub fn pass() -> String {
    println!("passing!");
    format!("")
}