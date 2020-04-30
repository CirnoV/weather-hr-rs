use serde::{Deserialize, Serialize};

const AIR_GANGWON_URI: &str = "http://www.airgangwon.go.kr/include/php/json/json_RealCityData.php";

#[derive(Clone, Debug, Serialize)]
pub struct Particulates {
    pm10: Option<f64>,
    pm25: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AirGangwon {
    date: String,
    time: String,
    realcitydata: Vec<Realcitydata>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Realcitydata {
    cityname: String,
    pm10: ::serde_json::Value,
    pm25: ::serde_json::Value,
}

pub async fn get_particulates() -> Particulates {
    let airgangwon: AirGangwon = match reqwest::get(AIR_GANGWON_URI).await {
        Ok(res) => res.json().await.unwrap(),
        Err(_) => {
            return Particulates {
                pm10: None,
                pm25: None,
            }
        }
    };
    let inje: &Realcitydata = airgangwon
        .realcitydata
        .iter()
        .find(|&x| x.cityname == "인제군")
        .unwrap();
    let pm10: Option<f64> = inje.pm10.to_string().parse().ok();
    let pm25: Option<f64> = inje.pm25.to_string().parse().ok();

    Particulates { pm10, pm25 }
}

#[tokio::test]
async fn test_get_particulates() {
    get_particulates().await;
}
