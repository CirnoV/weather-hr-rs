#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::{env, sync::Arc};

use chrono::prelude::*;
use serde::Serialize;
use tokio::sync::RwLock;

mod filters;
mod handlers;
mod request;
mod utils;
mod weather;

pub type WeatherData = Arc<RwLock<Vec<weather::Weather>>>;

#[derive(Debug, Serialize)]
struct WeatherJson {
    current: usize,
    length: usize,
    data: Option<weather::Weather>,
}

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "weather=info");
    }
    pretty_env_logger::init();

    let weather: WeatherData = Arc::new(RwLock::new(Vec::new()));
    let initial_date_time = Local.ymd(1970, 1, 1).and_hms(0, 0, 0).timestamp_millis();
    let last_updated = Arc::new(RwLock::new(initial_date_time));
    let routes = filters::weather(weather, last_updated);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
