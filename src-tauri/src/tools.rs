use crate::globals::{
    get_ip_api_key, get_opencage_key, get_reqwest_client, get_weather_api_user_agent,
};
use base64::prelude::{Engine as _, BASE64_STANDARD_NO_PAD};
use chrono::prelude::Local;
use image::{
    codecs::png::PngEncoder, imageops::resize, imageops::FilterType::Triangle, ColorType::Rgba8,
    ImageBuffer, ImageEncoder, Rgba,
};
use scrap::{Capturer, Display};
use serde_json::Value;
use std::{fs::File, io::ErrorKind::WouldBlock, path::Path, thread::sleep, time::Duration};
use urlencoding::encode;

#[cfg(target_os = "windows")]
mod windows_specific {
    pub use std::os::windows::ffi::{OsStrExt, OsStringExt};
    pub use winapi::um::winbase::{GlobalLock, GlobalUnlock};
    pub use winapi::um::winuser::{
        CloseClipboard, GetClipboardData, OpenClipboard, CF_UNICODETEXT,
    };
}

pub async fn get_location_coordinates(location: &str) -> String {
    println!("getting {} coordinates!", location);
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
                return format!(
                    "lat: {}, lng: {}",
                    coordinates["results"][0]["geometry"]["lat"],
                    coordinates["results"][0]["geometry"]["lat"]
                )
            }
            Err(e) => return format!("Unable to parse response: {}", e),
        },
        Err(e) => format!("Request to get user's coordinates failed: {}", e),
    }
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
                                                        // println!("{}: {}\n", day_forecast["name"], day_forecast["detailedForecast"]);
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
                                                        return format!(
                                                            "Couldn't make day into an object."
                                                        )
                                                    }
                                                }
                                            }
                                            the_forecast
                                        }
                                        _ => return format!("No forecast in response."),
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

pub async fn get_user_coordinates() -> String {
    println!("getting user's location!");
    let user_coordinates_result = get_reqwest_client()
        .get(format!("https://ipapi.co/json/?key={}", get_ip_api_key()))
        .send()
        .await;

    match user_coordinates_result {
        Ok(user_coordinates_response) => match user_coordinates_response.json::<Value>().await {
            Ok(user_coordinates) => {
                return format!(
                    "lat: {}, lng: {}",
                    user_coordinates["latitude"], user_coordinates["longitude"]
                )
            }
            Err(e) => return format!("Unable to parse response: {}", e),
        },
        Err(e) => format!("Request to get user's coordinates failed: {}", e),
    }
}

#[cfg(target_os = "macos")]
pub async fn get_clipboard_text() -> String {
    todo!();
}

#[cfg(target_os = "windows")]
pub fn get_clipboard_text() -> String {
    println!("getting clipboard text! (windows)");
    unsafe {
        // try to open clipboard
        if windows_specific::OpenClipboard(ptr::null_mut()) == 0 {
            "ERROR: Unable to open clipboard".to_string();
        }

        // check if there is clipboard data
        let clipboard_data = windows_specific::GetClipboardData(CF_UNICODETEXT);
        if clipboard_data.is_null() {
            "ERROR: No clipboard data".to_string();
            windows_specific::CloseClipboard();
        }

        // make sure it doesn't change as we get its value
        let text_ptr = windows_specific::GlobalLock(clipboard_data) as *const u16;
        if text_ptr.is_null() {
            "ERROR: Unable to lock global memory".to_string();
            windows_specific::CloseClipboard();
        }

        // collect data
        let text_slice = std::slice::from_raw_parts(text_ptr, {
            let mut len = 0;
            while *text_ptr.offset(len) != 0 {
                len += 1;
            }
            len as usize
        });

        // covert text slice to String
        let selected_text = String::from_utf16_lossy(
            &std::ffi::OsString::from_wide(text_slice)
                .encode_wide()
                .collect::<Vec<_>>(),
        );

        // release lock and close clipboard
        windows_specific::GlobalUnlock(clipboard_data);
        windows_specific::CloseClipboard();

        selected_text
    }
}

pub async fn get_screenshot() -> String {
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
        println!(
            "first 10: {:?}\nlast 10: {:?}",
            base64_image.get(..10),
            base64_image.get(base64_image.len() - 10..)
        );

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

pub fn get_time() -> String {
    println!("getting time!");
    format!("{:#?}", Local::now())
}

pub fn pass() -> String {
    println!("passing!");
    format!("")
}
