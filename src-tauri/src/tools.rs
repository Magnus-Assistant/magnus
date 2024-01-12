use crate::globals::{
    get_ip_api_key, get_opencage_key, get_reqwest_client, get_weather_api_user_agent,
};
use base64::prelude::{Engine as _, BASE64_STANDARD_NO_PAD};
use chrono::prelude::Local;
use clipboard::{ClipboardContext, ClipboardProvider};
use image::{
    codecs::png::PngEncoder, imageops::resize, imageops::FilterType::Triangle, ColorType::Rgba8,
    ImageBuffer, ImageEncoder, Rgba,
};
use scrap::{Capturer, Display};
use serde_json::Value;
use std::{
    fs::File, io::ErrorKind::WouldBlock, path::Path, thread::sleep, time::Duration
};
use urlencoding::encode;

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
                format!(
                    "lat: {}, lng: {}",
                    coordinates["results"][0]["geometry"]["lat"],
                    coordinates["results"][0]["geometry"]["lat"]
                )
            }
            Err(e) => format!("Unable to parse response: {}", e),
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

pub async fn get_user_coordinates() -> String {
    println!("getting user's location!");
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
pub fn get_clipboard_text() -> String {
    let mut clipboard = ClipboardContext::new().unwrap();
    match clipboard.get_contents() {
        Ok(text) => text,
        Err(error) => {
            panic!("Error getting clipboard contents: {}", error);
        },
    }
}

// returns a string representation of a base64 screenshot of the primary display
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

pub fn get_system_report() -> String {
    use sysinfo::{
        Components, Disks, Networks, System,
    };
    
    // Please note that we use "new_all" to ensure that all list of
    // components, network interfaces, disks and users are already
    // filled!
    let mut sys = System::new_all();
    
    // First we update all information of our `System` struct.
    sys.refresh_all();
    
    println!("=> system:");
    // RAM and swap information:
    println!("total memory: {} bytes", sys.total_memory());
    println!("used memory : {} bytes", sys.used_memory());
    println!("total swap  : {} bytes", sys.total_swap());
    println!("used swap   : {} bytes", sys.used_swap());
    
    // Display system information:
    println!("System name:             {:?}", System::name());
    println!("System kernel version:   {:?}", System::kernel_version());
    println!("System OS version:       {:?}", System::os_version());
    println!("System host name:        {:?}", System::host_name());
    
    // Number of CPUs:
    println!("NB CPUs: {}", sys.cpus().len());
    
    // Display processes ID, name na disk usage:
    for (pid, process) in sys.processes() {
        println!("[{pid}] {} {:?}", process.name(), process.disk_usage());
    }
    
    // We display all disks' information:
    println!("=> disks:");
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        println!("{disk:?}");
    }
    
    // Network interfaces name, data received and data transmitted:
    let networks = Networks::new_with_refreshed_list();
    println!("=> networks:");
    for (interface_name, data) in &networks {
        println!("{interface_name}: {}/{} B", data.received(), data.transmitted());
    }
    
    // Components temperature:
    let components = Components::new_with_refreshed_list();
    println!("=> components:");
    for component in &components {
        println!("{component:?}");
    }

    return "".to_string();
}

pub fn get_time() -> String {
    println!("getting time!");
    format!("{:#?}", Local::now())
}

pub fn pass() -> String {
    println!("passing!");
    String::new()
}
