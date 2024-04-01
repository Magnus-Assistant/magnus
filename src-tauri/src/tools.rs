use crate::globals::{
    get_ip_api_key, get_opencage_key, get_reqwest_client, get_weather_api_user_agent,
};
use crate::permissions::{check, Permission, Permission::*};
use base64::prelude::{Engine as _, BASE64_STANDARD_NO_PAD};
use chrono::prelude::Local;
use clipboard::{ClipboardContext, ClipboardProvider};
use image::{
    codecs::png::PngEncoder, imageops::resize, imageops::FilterType::Triangle, ColorType::Rgba8,
    ImageBuffer, ImageEncoder, Rgba,
};
use lazy_static::lazy_static;
use scrap::{Capturer, Display};
use serde_json::{Map, Value};
use std::{
    fs::File, future::Future, io::ErrorKind::WouldBlock, path::Path, pin::Pin, sync::Arc, thread::sleep, time::Duration
};
use urlencoding::encode;

type SyncAction = dyn Fn(Map<String, Value>) -> String + Send + Sync;
type AsyncAction = dyn Fn(Map<String, Value>) -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync;

pub enum Action {
    Sync(Arc<SyncAction>),
    Async(Arc<AsyncAction>),
}

pub struct Tool {
    pub action: Action,
    pub description: String,
    pub permissions: Option<Vec<Permission>>
}

impl Tool {
    pub fn new_sync<F>(action: F, description: String, permissions: Option<Vec<Permission>>) -> Self
    where
        F: Fn(Map<String, Value>) -> String + Send + Sync + 'static,
    {
        Tool {
            action: Action::Sync(Arc::new(action)),
            description: description,
            permissions: permissions
        }
    }

    pub fn new_async<F, Fut>(action: F, description: String, permissions: Option<Vec<Permission>>) -> Self
    where
        F: Fn(Map<String, Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        Tool {
            action: Action::Async(Arc::new(move |args| Box::pin(action(args)))),
            description: description,
            permissions: permissions
        }
    }

    pub async fn execute(&self, args: Map<String, Value>) -> String {
        // check if all permissions are satisfied
        if let Some(permissions) = &self.permissions {
            if let Some(result) = check(permissions.to_vec()) {
                println!("got result !!! {}", result.clone());
                return result
            }
        }

        // TODO: emit the description to the frontend
        println!("**{}...**", &self.description);

        match &self.action {
            Action::Sync(action) => action(args),
            Action::Async(action) => action(args).await,
        }
    }
}

lazy_static! {
    pub static ref CLIPBOARD: Tool = Tool::new_sync(get_clipboard_text, "Peeking at your clipboard".to_string(), Some(vec![Clipboard]));
    pub static ref FORECAST: Tool = Tool::new_async(get_forecast, "Checking the radar".to_string(), None);
    pub static ref LOCATION_COORDINATES: Tool = Tool::new_async(get_location_coordinates, "Looking at the map".to_string(), None);
    pub static ref SCREENSHOT: Tool = Tool::new_async(get_screenshot, "Peeking at your screen".to_string(), Some(vec![Screenshot]));
    pub static ref TIME: Tool = Tool::new_sync(get_time, "Checking wrist watch".to_string(), None);
    pub static ref USER_COORDINATES: Tool = Tool::new_async(get_user_coordinates, "Accessing your location".to_string(), Some(vec![Location]));
}

pub async fn get_location_coordinates(args: Map<String, Value>) -> String {
    let location = args.get("location").unwrap().as_str().unwrap();

    let coordinates_result = get_reqwest_client()
        .get(format!(
            "https://api.opencagedata.com/geocode/v1/json?key={}&q={}",
            get_opencage_key(),
            encode(location)
        ))
        .send()
        .await;
    
    match coordinates_result {
        Ok(coordinates_response) => match coordinates_response.json::<Value>().await {
            Ok(coordinates) => {
                format!(
                    "lat: {}, lng: {}",
                    coordinates["results"][0]["geometry"]["lat"],
                    coordinates["results"][0]["geometry"]["lng"]
                )
            }
            Err(e) => format!("Unable to parse response: {}", e),
        },
        Err(e) => format!("Request to get user's coordinates failed: {}", e),
    }
}

pub async fn get_forecast(args: Map<String, Value>) -> String {
    let lat = &args.get("latitude").unwrap().to_string();
    let lng = &args.get("longitude").unwrap().to_string();
    let n_days = &args.get("n_days").unwrap().to_string();

    let weather_result = get_reqwest_client()
        .get(format!("https://api.weather.gov/points/{},{}", lat, lng))
        .header("User-Agent", get_weather_api_user_agent())
        .send()
        .await;

    match weather_result {
        Ok(weather_response) => {
            match weather_response.json::<Value>().await {
                Ok(weather) => {
                    let forecast_url = weather["properties"]["forecast"]
                        .to_string()
                        .trim_matches('"')
                        .to_string();
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
                                                        the_forecast.push_str(
                                                            format!(
                                                                "{}: {}\n",
                                                                day_forecast["name"],
                                                                day_forecast["detailedForecast"]
                                                            )
                                                            .as_str(),
                                                        );
                                                    }
                                                    _ => {
                                                        return "Couldn't make day into an object."
                                                            .to_string()
                                                    }
                                                }
                                            }
                                            the_forecast
                                        }
                                        _ => "No forecast in response.".to_string(),
                                    }
                                }
                                Err(e) => format!("Unable to parse forecast response: {}", e),
                            }
                        }
                        Err(e) => format!("Request to get forecast failed: {}", e),
                    }
                }
                Err(e) => format!("Unable to parse weather response: {}", e),
            }
        }
        Err(e) => format!("Request to get weather failed: {}", e),
    }
}

pub async fn get_user_coordinates(_: Map<String, Value>) -> String {
    let user_coordinates_result = get_reqwest_client()
        .get(format!("https://ipapi.co/json/?key={}", get_ip_api_key()))
        .send()
        .await;

    match user_coordinates_result {
        Ok(user_coordinates_response) => match user_coordinates_response.json::<Value>().await {
            Ok(user_coordinates) => {
                format!(
                    "lat: {}, lng: {}",
                    user_coordinates["latitude"], user_coordinates["longitude"]
                )
            }
            Err(e) => format!("Unable to parse response: {}", e),
        },
        Err(e) => format!("Request to get user's coordinates failed: {}", e),
    }
}

// returns the contents of the systems clipboard
pub fn get_clipboard_text(_: Map<String, Value>) -> String {
    let mut clipboard = ClipboardContext::new().unwrap();
    match clipboard.get_contents() {
        Ok(text) => text,
        Err(error) => {
            panic!("Error getting clipboard contents: {}", error);
        },
    }
}

// returns a string representation of a base64 screenshot of the primary display
pub async fn get_screenshot(_: Map<String, Value>) -> String {
    let display = Display::primary().expect("Couldn't find primary display.");
    let width: u32 = display.width().try_into().unwrap();
    let height: u32 = display.height().try_into().unwrap();
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

    loop {
        // wait for a frame
        let buffer = match capturer.frame() {
            Ok(buffer) => buffer,
            Err(e) if e.kind() == WouldBlock => {
                sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => panic!("Error: {}", e),
        };

        // convert the image data to an image
        let img = ImageBuffer::from_fn(width, height, |x, y| {
            let index = 4 * (y * width + x) as usize;
            let data = &buffer[index..index + 4];
            Rgba([data[0], data[1], data[2], data[3]])
        });

        // resize the image -> smallest side (height) must be < 768 px for high res model, and < 512 for low res model
        let new_height: u32 = 768; // or 512
        let new_width: u32 = new_height * width / height;
        let resized_img = resize(&img, new_width, new_height, Triangle);

        // save the image into a new vec
        let mut bytes: Vec<u8> = Vec::new();
        PngEncoder::new(&mut bytes)
            .write_image(&resized_img, new_width, new_height, Rgba8)
            .unwrap();

        // encode the image data to base64
        let base64_image = &BASE64_STANDARD_NO_PAD.encode(&bytes);

        // write image for now, for viewing purposes. this will be removed later
        let path = Path::new("C:/Users/schre/Projects/screenshot.png");
        let file = File::create(path).expect("Couldn't create output file.");
        PngEncoder::new(file)
            .write_image(&resized_img, new_width, new_height, Rgba8)
            .expect("Couldn't encode frame.");

        // only need one frame
        return base64_image.to_string();
    }
}

pub fn get_time(_: Map<String, Value>) -> String {
    format!("{:#?}", Local::now())
}
