use reqwest::{Client, Response};

pub(super) async fn fetch(client: &Client, path: &str) -> reqwest::Result<Response> {
    let url = format!("https://skolmaten.se/api/4/{}", path);

    client
        .get(&url)
        .header("API-Version", "4.0")
        .header("Client-Token", "web")
        .header("Client-Version-Token", "web")
        .header("Locale", "sv_SE")
        .send()
        .await
}
