use reqwest;
use crate::globals::get_reqwest_client;

//const DOMAIN: &str = "https://magnusbackend.azurewebsites.net/";

const DOMAIN: &str = "http://localhost:3000/api";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct User {
    user_id: String,
    username: String,
    email: String,
    created_at: String
}

pub async fn get_user(id: String) -> Result<User, Box<dyn std::error::Error>> {
    let url: String = format!("{}/user/{}", DOMAIN, id);

    let response = reqwest::get(url)
        .await?
        .json::<User>()
        .await?;

    return Ok(response);

}

pub async fn add_user(user: User) -> Result<(), Box<dyn std::error::Error>> {
let url: String = format!("{}/user", DOMAIN);

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
