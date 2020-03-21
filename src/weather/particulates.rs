use serde::{Deserialize, Serialize};

const AIR_GANGWON_URI: &str = "http://www.airgangwon.go.kr/include/php/json/json_RealCityData.php";

#[derive(Debug, Serialize)]
pub struct Particulates {
    pm10: Option<f64>,
    pm25: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AirGangwon {
    date: String,
    time: String,
    realcitydata: Vec<Realcitydata>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Realcitydata {
    cityname: String,
    pm10: ::serde_json::Value,
    pm25: ::serde_json::Value,
}

pub async fn get_particulates() -> Particulates {
    let airgangwon: AirGangwon = reqwest::get(AIR_GANGWON_URI)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let inje: &Realcitydata = airgangwon
        .realcitydata
        .iter()
        .find(|&x| x.cityname == "인제군")
        .unwrap();
    let pm10: Option<f64> = inje.pm10.to_string().parse().ok();
    let pm25: Option<f64> = inje.pm25.to_string().parse().ok();

    Particulates { pm10, pm25 }
}
