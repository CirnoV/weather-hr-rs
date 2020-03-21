#[macro_use]
extern crate lazy_static;
extern crate regex;

mod request;
mod utils;
mod weather;

#[tokio::main]
async fn main() {
    weather::get_weather().await;
}
