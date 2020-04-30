use std::{convert::Infallible, sync::Arc};

use chrono::{prelude::*, Duration};
use tokio::sync::RwLock;

pub async fn get_weathers(
    weather: super::WeatherData,
    last_updated: Arc<RwLock<i64>>,
    page: Option<usize>,
) -> Result<impl warp::Reply, Infallible> {
    let (should_update, now) = {
        let last_updated_read = last_updated.read().await;
        let update_time = Local.timestamp_millis(*last_updated_read) + Duration::minutes(3);
        let now = Local::now();
        (update_time <= now, now)
    };
    if should_update {
        {
            *last_updated.write().await = now.timestamp_millis();
        }
        update_weathers(weather.clone(), now).await;
    }
    {
        let weather_read = weather.read().await;
        let len = weather_read.len();
        let pos: usize = match page {
            Some(pos) => pos,
            None => match len > 0 {
                true => len - 1,
                false => 0,
            },
        };
        let length = weather_read.len();
        let weather = match weather_read.get(pos) {
            Some(weather) => Some(weather.clone()),
            None => None,
        };
        let result = super::WeatherJson {
            data: weather,
            length,
            current: pos,
        };
        Ok(warp::reply::json(&result))
    }
}

async fn update_weathers(weather: super::WeatherData, now: DateTime<Local>) {
    let weather_data = super::weather::get_weather(now.timestamp_millis()).await;
    {
        let mut weather_write = weather.write().await;
        match weather_write.last() {
            Some(last_weather) => {
                let last_hr = Local.timestamp_millis(last_weather.timestamp).hour();
                let current_hr = now.hour();
                if last_hr == current_hr {
                    weather_write.pop();
                }
            }
            None => (),
        };
        weather_write.push(weather_data);
        let inner: Vec<super::weather::Weather> = weather_write
            .clone()
            .into_iter()
            .filter(|w| w.timestamp > (now - Duration::days(3)).timestamp_millis())
            .collect();
        *weather_write = inner;
    }
}
