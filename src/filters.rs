use std::sync::Arc;

use tokio::sync::Mutex;
use warp::Filter;

pub fn weather(
    weather: super::WeatherData,
    last_updated: Arc<Mutex<i64>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    index(weather.clone(), last_updated.clone()).or(with_params(weather, last_updated))
}

pub fn with_weather(
    weather: super::WeatherData,
    last_updated: Arc<Mutex<i64>>,
) -> warp::filters::BoxedFilter<(super::WeatherData, Arc<Mutex<i64>>)> {
    warp::any()
        .and(warp::any().map(move || weather.clone()))
        .and(warp::any().map(move || last_updated.clone()))
        .boxed()
}

pub fn index(
    weather: super::WeatherData,
    last_updated: Arc<Mutex<i64>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::any()
        .and(warp::get())
        .and(with_weather(weather, last_updated))
        .and_then(|weather, last_updated| {
            super::handlers::get_weathers(weather, last_updated, None)
        })
}

pub fn with_params(
    weather: super::WeatherData,
    last_updated: Arc<Mutex<i64>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!(usize)
        .and(warp::get())
        .and(with_weather(weather, last_updated))
        .and_then(|page, weather, last_updated| {
            super::handlers::get_weathers(weather, last_updated, Some(page))
        })
}
