use crate::globals::{
    get_ip_api_key, get_opencage_key, get_reqwest_client, get_weather_api_user_agent,
};
use crate::settings::{check_permissions, Permission, Permission::*};
use crate::{Payload, APP_HANDLE};
use anyhow::Context;
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
    fs::File, future::Future, io::ErrorKind::WouldBlock, path::Path, pin::Pin, sync::Arc,
    thread::sleep, time::Duration,
};
use tauri::Manager;
use urlencoding::encode;

/*
All actions MUST:
1. Take one arg, a serde_json::Map with String keys and serde_json::Value values. If the action doesn't need args,
then name the arg in the function signature as an underscore.

Traits:
Send - this is needed so that the Tool can be transferred to another thread
Sync - this is needed so that the Tool can be access from multiple threads
Without these two traits, we are unable to define the Tools as public static references and use them in assistant.rs

AsyncAction returns a Future that results in a String, which must be wrapped in a Box since the size of the Future in
memory is not statically known, it can vary. Wrapping the Future in a Box creates the Future in heap-space, where the
size is able vary. This Box is then wrapped in a Pin which just ensures that the Box doesn't get moved around in memory.
*/
type SyncAction = dyn Fn(Map<String, Value>) -> anyhow::Result<String> + Send + Sync;
type AsyncAction = dyn Fn(Map<String, Value>) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>
    + Send
    + Sync;

pub enum Action {
    Sync(Arc<SyncAction>),
    Async(Arc<AsyncAction>),
}

pub struct Tool {
    pub action: Action,      // the function that runs when Magnus uses the tool
    pub description: String, // the message that is displayed on the frontend when Magnus uses the tool
    pub permissions: Option<Vec<Permission>>, // the list of Permissions needed to execute the tool
}

impl Tool {
    pub fn new_sync<F>(action: F, description: String, permissions: Option<Vec<Permission>>) -> Self
    where
        F: Fn(Map<String, Value>) -> anyhow::Result<String> + Send + Sync + 'static,
    {
        Tool {
            action: Action::Sync(Arc::new(action)),
            description: description,
            permissions: permissions,
        }
    }

    pub fn new_async<F, Fut>(
        action: F,
        description: String,
        permissions: Option<Vec<Permission>>,
    ) -> Self
    where
        F: Fn(Map<String, Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<String>> + Send + 'static,
    {
        Tool {
            action: Action::Async(Arc::new(move |args| Box::pin(action(args)))),
            description: description,
            permissions: permissions,
        }
    }

    pub async fn execute(&self, args: Map<String, Value>) -> anyhow::Result<String> {
        // check if all permissions are satisfied
        if let Some(permissions) = &self.permissions {
            if let Some(result) = check_permissions(permissions.to_vec()) {
                println!("got result !!! {}", result.clone());
                return Ok(result);
            }
        }

        // TODO: emit the description to the frontend
        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            let narration = format!("*{}...*", &self.description);
            let _ = app_handle.emit_all("action", Payload { message: narration });
        }
        println!("**{}...**", &self.description);

        match &self.action {
            Action::Sync(action) => Ok(action(args).context("Failed to run sync action")?),
            Action::Async(action) => action(args).await.context("Failed to run async action"),
        }
    }
}

// create Tools here to be exposed in assistant.rs
lazy_static! {
    pub static ref CLIPBOARD: Tool = Tool::new_sync(
        get_clipboard_text,
        "Peeking at your clipboard".to_string(),
        Some(vec![Clipboard])
    );
    pub static ref FORECAST: Tool =
        Tool::new_async(get_forecast, "Checking the radar".to_string(), None);
    pub static ref LOCATION_COORDINATES: Tool = Tool::new_async(
        get_location_coordinates,
        "Looking at the map".to_string(),
        None
    );
    pub static ref SCREENSHOT: Tool = Tool::new_async(
        get_screenshot,
        "Peeking at your screen".to_string(),
        Some(vec![Screenshot])
    );
    pub static ref TIME: Tool = Tool::new_sync(get_time, "Checking wrist watch".to_string(), None);
    pub static ref USER_COORDINATES: Tool = Tool::new_async(
        get_user_coordinates,
        "Accessing your location".to_string(),
        Some(vec![Location])
    );
}

pub async fn get_location_coordinates(args: Map<String, Value>) -> anyhow::Result<String> {
    let location = args.get("location").unwrap().as_str().unwrap();

    let coordinates_result = get_reqwest_client()
        .get(format!(
            "https://api.opencagedata.com/geocode/v1/json?key={}&q={}",
            get_opencage_key(),
            encode(location)
        ))
        .send()
        .await
        .context("Failed to get users location coordinates")?
        .json::<Value>()
        .await
        .context("Failed to parse coordinate values in response")?;

    let lat = &coordinates_result["results"][0]["geometry"]["lat"];
    let lng = &coordinates_result["results"][0]["geometry"]["lng"];

    Ok(format!("lat: {}, lng: {}", lat, lng))
}

pub async fn get_forecast(args: Map<String, Value>) -> anyhow::Result<String> {
    // parse the incoming forcast arguments
    let lat = &args
        .get("latitude")
        .context("Latitude Missing")?
        .to_string();
    let lng = &args
        .get("longitude")
        .context("Longitude Missing")?
        .to_string();
    let n_days = &args
        .get("n_days")
        .context("N Days Missing")?
        .as_i64()
        .context("Failed to parse n_days")?
        * 2; // multiply by 2 because we receive the forcast in half days

    // create the weather URL
    let weather_url = format!("https://api.weather.gov/points/{},{}", lat, lng);

    // request for and parse weather results
    let weather_result = get_reqwest_client()
        .get(&weather_url)
        .header("User-Agent", get_weather_api_user_agent())
        .send()
        .await
        .context("Failed to request weather data")?
        .json::<Value>()
        .await
        .context("Failed to parse parse weather json response")?;

    // create initial values
    let forecast_url = weather_result["properties"]["forecast"].as_str();

    // create an initial vec that can hold the days later if they exist in the match
    let days_vec = vec![Value::Null];
    let days = Some(&days_vec);
    let mut the_forecast = String::new();

    // guard against null values in these two vars
    match (forecast_url, days) {
        // send and parse the forecast response
        (Some(forecast_url), Some(_days)) => {
            let forecast_response = get_reqwest_client()
                .get(forecast_url)
                .header("User-Agent", get_weather_api_user_agent())
                .send()
                .await
                .context("Failed to request forecast data")?
                .json::<Value>()
                .await
                .context("Failed to parse forecast response")?;

            // save days to vec
            if let Some(days) = forecast_response["properties"]["periods"].as_array() {
                for day in 0..n_days as usize {
                    // change to usize so we can use it for indexing
                    let name = days[day]
                        .get("name")
                        .and_then(Value::as_str)
                        .context("Missing forecast day name")?;
                    let detailed_forecast = days[day]
                        .get("detailedForecast")
                        .and_then(Value::as_str)
                        .context("Missing detailed forecast")?;
                    the_forecast.push_str(&format!("{}: {}\n", name, detailed_forecast));
                }
            } else {
                // if we dont have any days information return an error to the assistant
                return Ok(
                    "Error recieving weather information. Please try again later".to_string(),
                );
            }

            // all is well, return the forecast
            Ok(the_forecast)
        }
        // if we get any value other than the ones we haven't caught above, error
        (_, _) => Ok("Error recieving weather information. Please try again later".to_string()),
    }
}

pub async fn get_user_coordinates(_: Map<String, Value>) -> anyhow::Result<String> {
    let user_coordinates_result = get_reqwest_client()
        .get(format!("https://ipapi.co/json/?key={}", get_ip_api_key()))
        .send()
        .await
        .context("Failed to send request for user coordinates")?
        .json::<Value>()
        .await
        .context("Failed to parse user coordinates")?;

    // if we successfully hit the lat and lng api we shouldnt have to worry about this failing
    let lat = &user_coordinates_result["latitude"];
    let lng = &user_coordinates_result["longitude"];

    Ok(format!("lat: {}, lng: {}", lat, lng))
}

// returns the contents of the systems clipboard
pub fn get_clipboard_text(_: Map<String, Value>) -> anyhow::Result<String> {
    // map errors because anyhows "Error" doesn't direct support box errors
    let mut clipboard = ClipboardContext::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let contents = clipboard
        .get_contents()
        .map_err(|e| anyhow::anyhow!(e.to_string()))
        .context("Failed to grab string contents of clipboard")?;

    Ok(contents)
}

// returns a string representation of a base64 screenshot of the primary display
pub async fn get_screenshot(_: Map<String, Value>) -> anyhow::Result<String> {
    let display = Display::primary().context("Failed to gather display information")?;
    let width: u32 = display
        .width()
        .try_into()
        .context("Failed to convert width to u32")?;
    let height: u32 = display
        .height()
        .try_into()
        .context("Failed to convert height to u32")?;
    let mut capturer = Capturer::new(display).context("Unable to start capture.")?;

    loop {
        // wait for a frame
        let buffer = match capturer.frame() {
            Ok(buffer) => buffer,
            Err(e) if e.kind() == WouldBlock => {
                sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => panic!("Error: {}", e), // Leaving this as is for now while I convert things to anyhow results
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
        let file = File::create(path).context("Failed to created output file for screenshot")?;
        PngEncoder::new(file)
            .write_image(&resized_img, new_width, new_height, Rgba8)
            .context("Failed to encode frame")?;

        // only need one frame
        return Ok(base64_image.to_string());
    }
}

pub fn get_time(_: Map<String, Value>) -> anyhow::Result<String> {
    Ok(format!("{:#?}", Local::now()))
}
