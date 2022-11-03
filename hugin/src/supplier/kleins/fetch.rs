use reqwest::{header::USER_AGENT, Client, IntoUrl, Response};

const UA: &str = "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:47.0) Gecko/20100101 Firefox/47.0";

pub(super) async fn fetch(client: &Client, url: impl IntoUrl) -> reqwest::Result<Response> {
    client.get(url).header(USER_AGENT, UA).send().await
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use super::*;

    #[tokio::test]
    async fn ok() {
        let res = fetch(&Client::new(), "https://www.kleinskitchen.se/skolor/")
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }
}
