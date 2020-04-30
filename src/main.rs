#[macro_use]
extern crate lazy_static;
extern crate regex;

use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use chrono::prelude::*;
use chrono::Duration;
use serde::Serialize;
use std::{io, sync::Mutex};

mod request;
mod utils;
mod weather;

type WeatherData = Vec<weather::Weather>;

#[derive(Debug, Serialize)]
struct WeatherJson {
    current: usize,
    length: usize,
    data: Option<weather::Weather>,
}

async fn update_weathers(now: DateTime<Local>, weathers: &web::Data<Mutex<WeatherData>>) {
    let weather_data = weather::get_weather(now.timestamp_millis()).await;
    {
        let mut weathers = weathers.lock().unwrap();
        match weathers.last() {
            Some(last_weather) => {
                let last_hr = Local.timestamp_millis(last_weather.timestamp).hour();
                let current_hr = now.hour();
                if last_hr == current_hr {
                    weathers.pop();
                }
            }
            None => (),
        };
        weathers.push(weather_data);
        let inner: Vec<weather::Weather> = weathers
            .clone()
            .into_iter()
            .filter(|w| w.timestamp > (now - Duration::days(3)).timestamp_millis())
            .collect();
        *weathers = inner;
    }
}

async fn get_weathers(
    pos: usize,
    weathers: &web::Data<Mutex<WeatherData>>,
    last_updated: &web::Data<Mutex<DateTime<Local>>>,
) -> WeatherJson {
    let update_time = *last_updated.lock().unwrap() + Duration::minutes(3);
    let now = Local::now();
    if update_time <= now {
        *last_updated.lock().unwrap() = now;
        update_weathers(now, &weathers).await;
    }
    let weather = weathers.lock().unwrap();
    let length = weather.len();
    let weather = match weather.get(pos) {
        Some(weather) => Some(weather.clone()),
        None => None,
    };
    let result = WeatherJson {
        data: weather,
        length,
        current: pos,
    };
    result
}

async fn index(
    weathers: web::Data<Mutex<WeatherData>>,
    last_updated: web::Data<Mutex<DateTime<Local>>>,
    _req: HttpRequest,
) -> HttpResponse {
    let pos = {
        let length = (*weathers.lock().unwrap()).len();
        match length > 0 {
            true => length - 1,
            false => 0,
        }
    };
    let result = get_weathers(pos, &weathers, &last_updated).await;
    HttpResponse::Ok().json(result)
}

async fn index_with_param(
    weathers: web::Data<Mutex<WeatherData>>,
    last_updated: web::Data<Mutex<DateTime<Local>>>,
    _req: HttpRequest,
    path: web::Path<(String,)>,
) -> HttpResponse {
    let length = { (*weathers.lock().unwrap()).len() };
    let pos: usize = match path.0.parse() {
        Ok(pos) => pos,
        Err(_) => match length > 0 {
            true => length - 1,
            false => 0,
        },
    };
    let result = get_weathers(pos, &weathers, &last_updated).await;
    HttpResponse::Ok().json(result)
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
            .service(web::resource("/{pos}").route(web::get().to(index_with_param)))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
