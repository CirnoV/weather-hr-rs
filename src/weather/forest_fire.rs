use serde::Serialize;

use crate::{
    request::request,
    utils::{elem_to_string, parse_elem},
};
use scraper::{ElementRef, Html, Selector};

const FOREST_FIRE_URI: &str = "http://forestfire.nifos.go.kr/mobile/jsp/fireGrade.jsp?cd=42&cdName=%EA%B0%95%EC%9B%90%EB%8F%84&subCd=42810&subCdName=%EC%9D%B8%EC%A0%9C%EA%B5%B0";

#[derive(Clone, Debug, Serialize)]
pub struct ForestFire {
    value: Option<f64>,
}

pub async fn get_forest_fire() -> ForestFire {
    let html: String = match request(FOREST_FIRE_URI).await {
        Ok(res) => res,
        Err(_) => return ForestFire { value: None },
    };
    let document = Html::parse_document(&html);
    let tr = Selector::parse("div.greenTable > table > tbody > tr").unwrap();
    let td = Selector::parse("td").unwrap();
    let tr: Vec<ElementRef> = document.select(&tr).collect();
    let value = tr.iter().find_map(|elem| {
        let td = elem.select(&td).collect::<Vec<_>>();
        let value: Option<f64> = match elem_to_string(&td[0]) == "인제군" {
            true => parse_elem(&td[2]),
            _ => None,
        };
        value
    });

    ForestFire { value }
}
