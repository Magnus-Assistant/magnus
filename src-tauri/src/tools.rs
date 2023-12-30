pub fn get_location_weather(location: &str) -> String {
    println!("getting weather of {}!", location);
    format!("Its currently 40 F in {}", location)
}

pub fn get_local_weather(latitude: &str, longitude: &str) -> String {
    println!("getting local weather!");
    format!("Weather of {}, {} is warm and sunny!", latitude, longitude)
}

pub fn get_user_location() -> String {
    println!("getting user location!");
    format!("latitude: 39.0997, longitude: -94.578331")
}

pub fn pass() -> String {
    println!("passing!");
    format!("")
}