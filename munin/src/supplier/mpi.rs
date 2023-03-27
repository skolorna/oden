use stor::menu::Supplier;

use crate::mashie::mashie_impl;

mashie_impl!("https://mpi.mashie.com", Supplier::Mpi);

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use time::OffsetDateTime;

    #[tokio::test]
    async fn sodra_latin() {
        let today = OffsetDateTime::now_utc().date();
        let days = super::list_days(
            &Client::new(),
            "c3c75403-6811-400a-96f8-a0e400c020ba",
            today..=today,
        )
        .await
        .unwrap();
    }
}
