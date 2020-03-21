use easy_scraper::Pattern;
use serde::Serialize;
use std::collections::BTreeMap;

use crate::request::request;

const FOREST_FIRE_URI: &str = "http://forestfire.nifos.go.kr/mobile/jsp/fireGrade.jsp?cd=42&cdName=%EA%B0%95%EC%9B%90%EB%8F%84&subCd=42810&subCdName=%EC%9D%B8%EC%A0%9C%EA%B5%B0";

#[derive(Debug, Serialize)]
pub struct ForestFire {
    value: Option<f64>,
}

pub async fn get_forest_fire() -> ForestFire {
    let value = {
        let document: String = request(FOREST_FIRE_URI).await.unwrap();
        let pat: Pattern = Pattern::new(
            r#"
        <div class="greenTable">
            <table>
                <tbody>
                    <tr>
                        <td>{{location}}</td>
                        <td>{{rank}}</td>
                        <td>{{value}}</td>
                    </tr>
                </tbody>
            </table>
        </div>
        "#,
        )
        .unwrap();

        let data: Vec<BTreeMap<String, String>> = pat.matches(&document);
        let inje = data.iter().find(|&x| x["location"] == "인제군").unwrap();
        let value: Option<f64> = inje["value"].parse::<f64>().ok();
        value
    };

    ForestFire { value }
}
