use reqwest::Client;
use lazy_static::lazy_static;

lazy_static! {
    static ref REQWEST_CLIENT: Client = Client::new();
}

pub fn get_reqwest_client() -> &'static Client {
    &REQWEST_CLIENT
}
