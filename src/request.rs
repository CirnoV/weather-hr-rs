pub async fn request(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let html: String = reqwest::get(url).await?.text().await?;

    Ok(html)
}
