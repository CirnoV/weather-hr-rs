#[macro_use]
extern crate lazy_static;
extern crate regex;

mod request;
mod utils;
mod weather;

use clipboard_win::Clipboard;

#[tokio::main]
async fn main() {
    let weather = weather::get_weather().await;
    let json = serde_json::to_string_pretty(&weather).unwrap();
    Clipboard::new().unwrap().set_string(&json).unwrap();
}
