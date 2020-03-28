#[macro_use]
extern crate lazy_static;
extern crate regex;

use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use chrono::prelude::*;
use chrono::Duration;
use std::{io, sync::Mutex};

mod request;
mod utils;
mod weather;

type WeatherData = Vec<weather::Weather>;

async fn index(
    weather: web::Data<Mutex<WeatherData>>,
    last_updated: web::Data<Mutex<DateTime<Local>>>,
    _req: HttpRequest,
) -> HttpResponse {
    let update_time = *last_updated.lock().unwrap() + Duration::minutes(3);
    let now = Local::now();
    if update_time <= now {
        *last_updated.lock().unwrap() = now;
        let weather_data = weather::get_weather(now.timestamp_millis()).await;
        {
            let mut weather = weather.lock().unwrap();
            weather.push(weather_data);
            let inner: Vec<weather::Weather> = weather
                .clone()
                .into_iter()
                .filter(|w| w.timestamp > (now - Duration::days(3)).timestamp_millis())
                .collect();
            *weather = inner;
        };
        println!("Updated, {}", now);
    }
    HttpResponse::Ok().json(&*weather.lock().unwrap())
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let weather = web::Data::new(Mutex::new(WeatherData::new()));
    let initial_date_time = Local.ymd(1970, 1, 1).and_hms(0, 0, 0);
    let last_updated = web::Data::new(Mutex::new(initial_date_time));

    HttpServer::new(move || {
        App::new()
            .app_data(weather.clone())
            .app_data(last_updated.clone())
            // enable logger
            .wrap(middleware::Logger::default())
            // register simple handler
            .service(web::resource("/").to(index))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
