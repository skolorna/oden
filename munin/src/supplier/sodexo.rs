use stor::menu::Supplier;

use crate::mashie::mashie_impl;

mashie_impl!("https://sodexo.mashie.com", Supplier::Sodexo);

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use time::{Duration, OffsetDateTime};

    use crate::supplier::ListDays;

    #[tokio::test]
    async fn list_menus() {
        let menus = super::list_menus(&Client::new()).await.unwrap();
        assert!(menus.len() > 100);
    }

    #[tokio::test]
    async fn list_days() {
        let first = OffsetDateTime::now_utc().date();
        let last = first + Duration::days(365);

        let ListDays { days, .. } = super::list_days(
            &Client::new(),
            "312dd0ae-3ebd-49d9-870e-abeb008c0e4b",
            first..=last,
        )
        .await
        .unwrap();

        assert!(days.len() > 5);

        for day in days {
            assert!(day.date >= first);
            assert!(day.date <= last);
        }
    }
}
