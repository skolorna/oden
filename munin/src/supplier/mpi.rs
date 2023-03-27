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
        let res = super::list_days(
            &Client::new(),
            "e4e189ac-f42d-4f82-89a8-aef300d00f33",
            today..=today,
        )
        .await
        .unwrap();

        assert_eq!(res.menu.title.unwrap(), "Södra Latins gymnasium");
    }
}
