use chrono::prelude::*;
use regex::Regex;
use scraper::ElementRef;
use std::str::FromStr;

pub fn elem_to_string(elem: &ElementRef) -> String {
    elem.text().collect::<String>()
}

pub fn parse_elem<T: FromStr>(elem: &ElementRef) -> Option<T> {
    elem_to_string(elem).parse::<T>().ok()
}

pub fn hhmm_to_timestamp_millis(text: &str) -> i64 {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(\d{2}):(\d{2})").unwrap();
    }
    let caps = RE.captures(&text).unwrap();
    let hour: u32 = caps.get(1).unwrap().as_str().parse().unwrap();
    let min: u32 = caps.get(2).unwrap().as_str().parse().unwrap();

    let now = Local::now();
    let local = Local
        .ymd(now.year(), now.month(), now.day())
        .and_hms(hour, min, 0);

    local.timestamp_millis()
}
