use super::WindDirection;
use crate::request::request;
use crate::utils::{elem_to_string, hhmm_to_timestamp_millis};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;

pub enum ForecastLocation {
    Wontong = 4281032000,
    Sudong = 4282034000,
}

impl ForecastLocation {
    pub fn to_string(&self) -> String {
        match self {
            ForecastLocation::Wontong => String::from("원통"),
            ForecastLocation::Sudong => String::from("수동"),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Forecast {
    location: String,
    timestamp: i64,
    temperature: Option<f64>,
    humidity: Option<f64>,
    wind_direction: WindDirection,
    wind_speed: Option<f64>,
}

fn parse_temperature(text: &str) -> Option<f64> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"([-+]?\d*\.?\d)℃$").unwrap();
    }
    let caps = RE.captures(text).unwrap();
    let temperature: Option<f64> = caps.get(1).unwrap().as_str().parse().ok();

    temperature
}

fn parse_humidity(text: &str) -> Option<f64> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"([-+]?\d*\.?\d)%$").unwrap();
    }
    let caps = RE.captures(text).unwrap();
    let humidity: Option<f64> = caps.get(1).unwrap().as_str().parse().ok();

    humidity
}

fn parse_wind(text: &str) -> (WindDirection, Option<f64>) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(.*) ([-+]?\d*\.?\d)m/s$").unwrap();
    }
    if RE.is_match(text) == false {
        return (
            WindDirection {
                angle: None,
                name: None,
            },
            None,
        );
    }
    let caps = RE.captures(text).unwrap();
    let wind_direction = caps.get(1).unwrap().as_str();
    let wind_speed: Option<f64> = caps.get(2).unwrap().as_str().parse().ok();

    (
        WindDirection {
            angle: None,
            name: Some(String::from(wind_direction)),
        },
        wind_speed,
    )
}

fn get_forecast_timestamp(document: &Html) -> i64 {
    let span = Selector::parse("p.MB5 > span:nth-child(2)").unwrap();
    let span = document.select(&span).next().unwrap();
    let text = elem_to_string(&span);
    let timestamp = hhmm_to_timestamp_millis(&text);

    timestamp
}

async fn get_forecast_data(location: ForecastLocation) -> Option<Forecast> {
    let location_name = location.to_string();
    let url = format!(
        "https://www.weather.go.kr/plus/rest/land/timeseries-body.jsp?code={}&unit=m%2Fs",
        location as u32
    );
    let html: String = match request(&url).await {
        Ok(res) => res,
        Err(_) => return None,
    };
    let document = Html::parse_document(&html);
    let dl = Selector::parse("div.now_weather1 > dl").unwrap();
    let dd = Selector::parse("dd").unwrap();

    let mut dl = document.select(&dl);
    let dd: Vec<ElementRef> = dl.next().unwrap().select(&dd).collect();
    let offset = match dd.len() {
        4 => 0,
        d if d > 4 => d - 4,
        _ => panic!(),
    };

    let timestamp = get_forecast_timestamp(&document);
    let temperature: Option<f64> = parse_temperature(&elem_to_string(&dd[offset]));
    let (wind_direction, wind_speed) = parse_wind(&elem_to_string(&dd[offset + 1]));
    let humidity: Option<f64> = parse_humidity(&elem_to_string(&dd[offset + 2]));

    Some(Forecast {
        location: location_name,
        timestamp,
        humidity,
        temperature,
        wind_direction,
        wind_speed,
    })
}

pub async fn get_forecast() -> Vec<Forecast> {
    let (wontong, sudong) = tokio::join!(
        get_forecast_data(ForecastLocation::Wontong),
        get_forecast_data(ForecastLocation::Sudong),
    );

    let forecast: Vec<Option<Forecast>> = vec![wontong, sudong];
    let forecast: Vec<Forecast> = forecast
        .into_iter()
        .filter_map(|x: Option<Forecast>| match x {
            Some(value) => Some(value),
            None => None,
        })
        .collect::<Vec<_>>();
    forecast
}
