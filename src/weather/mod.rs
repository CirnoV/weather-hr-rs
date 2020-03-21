mod aws;
mod forecast;
mod forest_fire;
mod particulates;

use chrono::prelude::*;
use serde::Serialize;

use aws::{get_aws_weather, AWS};
use forecast::{get_forecast, Forecast};
use forest_fire::{get_forest_fire, ForestFire};
use particulates::{get_particulates, Particulates};

#[derive(Debug, Serialize)]
pub struct WindDirection {
    name: Option<String>,
    angle: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct Weather {
    timestamp: i64,
    aws: Vec<AWS>,
    forecast: Vec<Forecast>,
    forest_fire: ForestFire,
    particulates: Particulates,
}

pub async fn get_weather() -> Weather {
    let aws = get_aws_weather();
    let forecast = get_forecast();
    let forest_fire = get_forest_fire();
    let particulates = get_particulates();

    let (aws, forecast, forest_fire, particulates) =
        tokio::join!(aws, forecast, forest_fire, particulates);

    let timestamp = Local::now().timestamp_millis();

    Weather {
        timestamp,
        aws,
        forecast,
        forest_fire,
        particulates,
    }
}
