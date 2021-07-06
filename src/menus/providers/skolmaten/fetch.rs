use reqwest::Client;
use serde::de::DeserializeOwned;

pub async fn fetch_json<T: DeserializeOwned>(client: &Client, path: &str) -> reqwest::Result<T> {
    let url = format!("https://skolmaten.se/api/4/{}", path);

    let res = client
        .get(&url)
        .header("API-Version", "4.0")
        .header("Client-Token", "web")
        .header("Client-Version-Token", "web")
        .header("Locale", "sv_SE")
        .send()
        .await?;

    let body = res.json::<T>().await?;

    Ok(body)
}
