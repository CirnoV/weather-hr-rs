use chrono::prelude::*;
use scraper::element_ref::ElementRef;
use scraper::{Html, Selector};
use serde::Serialize;

use super::WindDirection;
use crate::{
    request::request,
    utils::{elem_to_string, hhmm_to_timestamp_millis, parse_elem},
};

pub enum AWSLocation {
    Wontong = 321,
    Seohwa = 594,
    Jinburyeong = 595,
    Hyangnobong = 320,
    Inje = 211,
    Haean = 518,
}

impl AWSLocation {
    pub fn to_string(&self) -> String {
        match self {
            AWSLocation::Wontong => String::from("원통"),
            AWSLocation::Seohwa => String::from("서화"),
            AWSLocation::Jinburyeong => String::from("진부령"),
            AWSLocation::Hyangnobong => String::from("향로봉"),
            AWSLocation::Inje => String::from("인제"),
            AWSLocation::Haean => String::from("해안"),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AWS {
    location: String,
    source: String,
    data: Vec<AWSData>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AWSData {
    timestamp: i64,
    rainfall_per_day: Option<f64>,
    temperature: Option<f64>,
    wind_direction: WindDirection,
    wind_speed: Option<f64>,
    humidity: Option<f64>,
    sea_level_pressure: Option<f64>,
}

fn translate_cardinal_point(point: String) -> Option<String> {
    let point = point.to_uppercase();
    let name: Option<&str> = match point.as_str() {
        "N" => Some("북"),
        "NNE" => Some("북북동"),
        "NE" => Some("북동"),
        "ENE" => Some("동북동"),
        "E" => Some("동"),
        "ESE" => Some("동남동"),
        "SE" => Some("남동"),
        "SSE" => Some("남남동"),
        "S" => Some("남"),
        "SSW" => Some("남남서"),
        "SW" => Some("남서"),
        "WSW" => Some("서남서"),
        "W" => Some("서"),
        "WNW" => Some("서북서"),
        "NW" => Some("북서"),
        "NNW" => Some("북북서"),
        _ => None,
    };
    match name {
        Some(name) => Some(name.to_string()),
        None => None,
    }
}

async fn get_weather(location: AWSLocation) -> Option<AWS> {
    let location_name = location.to_string();
    let url = format!(
        "https://www.weather.go.kr/cgi-bin/aws/nph-aws_txt_min_cal_test?0&0&MINDB_1M&{}&a&M",
        location as u32
    );
    let html: String = match request(&url).await {
        Ok(res) => res,
        Err(_) => return None,
    };
    let document = Html::parse_document(&html);
    let tr = Selector::parse("table > tbody > tr > td > table > tbody > tr").unwrap();
    let td = Selector::parse("td").unwrap();

    let to_aws = |tr: ElementRef| -> AWSData {
        let td: Vec<ElementRef> = tr.select(&td).collect();
        let timestamp = hhmm_to_timestamp_millis(&elem_to_string(&td[0]));
        let rainfall_per_day: Option<f64> = parse_elem(&td[7]);
        let temperature: Option<f64> = parse_elem(&td[8]);
        let wind_direction: WindDirection = {
            let angle: Option<f64> = parse_elem(&td[12]);
            let name: Option<String> = translate_cardinal_point(elem_to_string(&td[13]));

            WindDirection { name, angle }
        };
        let wind_speed: Option<f64> = parse_elem(&td[14]);
        let humidity: Option<f64> = match td.get(15) {
            Some(elem) => parse_elem(elem),
            None => None,
        };
        let sea_level_pressure: Option<f64> = match td.get(16) {
            Some(elem) => parse_elem(elem),
            None => None,
        };

        AWSData {
            timestamp,
            rainfall_per_day,
            temperature,
            wind_direction,
            wind_speed,
            humidity,
            sea_level_pressure,
        }
    };

    let now = Local::now().timestamp_millis();
    let aws_data: Vec<AWSData> = document
        .select(&tr)
        .skip(1)
        .map(to_aws)
        .filter(|aws| aws.temperature != None || aws.timestamp > now)
        .collect();

    Some(AWS {
        location: location_name,
        source: url,
        data: aws_data,
    })
}

pub async fn get_aws_weather() -> Vec<AWS> {
    let (wontong, seohwa, jinburyeong, hyangnobong, inje, haean) = tokio::join!(
        get_weather(AWSLocation::Wontong),
        get_weather(AWSLocation::Seohwa),
        get_weather(AWSLocation::Jinburyeong),
        get_weather(AWSLocation::Hyangnobong),
        get_weather(AWSLocation::Inje),
        get_weather(AWSLocation::Haean),
    );

    let aws: Vec<Option<AWS>> = vec![wontong, seohwa, jinburyeong, hyangnobong, inje, haean];
    let aws: Vec<AWS> = aws
        .into_iter()
        .filter_map(|x: Option<AWS>| match x {
            Some(x) => Some(x),
            None => None,
        })
        .collect();
    aws
}
