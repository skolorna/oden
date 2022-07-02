pub mod scrape;

use chrono::NaiveDate;
use reqwest::{header::CONTENT_LENGTH, Client};
use select::document::Document;
use serde::Deserialize;
use tracing::instrument;

use crate::{
    errors::{Error, Result},
    menus::mashie::scrape::scrape_mashie_days,
    util::is_sorted,
    Day, Menu, MenuSlug,
};

use super::supplier::Supplier;

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize, Debug)]
pub struct MashieMenu {
    id: String,
    title: String,
    #[serde(rename(deserialize = "url"))]
    path: String,
}

impl MashieMenu {
    #[must_use]
    pub fn normalize(self, supplier: Supplier) -> Menu {
        let id = MenuSlug::new(supplier, self.id);
        Menu::new(id, self.title)
    }
}

#[instrument(err)]
pub async fn list_menus(host: &str) -> Result<Vec<MashieMenu>> {
    let client = Client::new();
    let res = client
        .post(&format!(
            "{}/public/app/internal/execute-query?country=se",
            host
        ))
        .header(CONTENT_LENGTH, "0")
        .send()
        .await?;

    let menus = res.json::<Vec<MashieMenu>>().await?;

    Ok(menus)
}

#[instrument(err)]
pub async fn query_menu(host: &str, menu_slug: &str) -> Result<MashieMenu> {
    let menus = list_menus(host).await?;
    let menu = menus
        .into_iter()
        .find(|m| m.id == menu_slug)
        .ok_or(Error::MenuNotFound)?;

    Ok(menu)
}

#[instrument(err, fields(%first, %last))]
pub async fn list_days(
    host: &str,
    menu_slug: &str,
    first: NaiveDate,
    last: NaiveDate,
) -> Result<Vec<Day>> {
    let menu = query_menu(host, menu_slug).await?;
    let url = format!("{}/{}", host, menu.path);
    let html = reqwest::get(&url).await?.text().await?;
    let doc = Document::from(html.as_str());
    let days: Vec<Day> = scrape_mashie_days(&doc)
        .into_iter()
        .filter(|day| day.is_between(first, last))
        .collect();

    debug_assert!(is_sorted(&days));

    Ok(days)
}

/// Automagically generate a Mashie client.
macro_rules! mashie_impl {
    ($host:literal, $supplier:expr) => {
        use chrono::NaiveDate;
        use $crate::errors::Result;
        use $crate::menus::{mashie, Menu};
        use $crate::Day;

        const HOST: &str = $host;

        pub async fn list_menus() -> Result<Vec<Menu>> {
            let menus = mashie::list_menus(HOST)
                .await?
                .into_iter()
                .map(|m| m.normalize($supplier))
                .collect();

            Ok(menus)
        }

        pub async fn list_days(
            menu_slug: &str,
            first: NaiveDate,
            last: NaiveDate,
        ) -> Result<Vec<Day>> {
            mashie::list_days(HOST, menu_slug, first, last).await
        }
    };
}

#[cfg(test)]
mod tests {
    use chrono::{offset, Duration};

    use super::*;

    #[tokio::test]
    async fn list_menus_test() {
        let menus = list_menus("https://sodexo.mashie.com").await.unwrap();

        assert!(menus.len() > 100);
    }

    #[tokio::test]
    async fn query_menu_test() {
        let menu = query_menu(
            "https://sodexo.mashie.com",
            "e8851c61-013b-4617-93d9-adab00820bcd",
        )
        .await
        .unwrap();

        assert_eq!(menu.title, "Södermalmsskolan, Södermalmsskolan");
        assert_eq!(menu.id, "e8851c61-013b-4617-93d9-adab00820bcd");

        assert!(query_menu("https://sodexo.mashie.com", "invalid")
            .await
            .is_err());
    }

    mod impl_test {
        use crate::util::is_sorted;

        use super::*;

        mashie_impl!("https://sodexo.mashie.com", Supplier::Sodexo);

        const MENU_SLUG: &str = "312dd0ae-3ebd-49d9-870e-abeb008c0e4b";

        #[tokio::test]
        async fn list_menus_test() {
            let menus = list_menus().await.unwrap();
            assert!(menus.len() > 100);
        }

        #[tokio::test]
        async fn list_days_test() {
            let first = offset::Utc::today().naive_utc();
            let last = first + Duration::days(365);

            let days = list_days(MENU_SLUG, first, last).await.unwrap();

            assert!(days.len() > 5);
            assert!(is_sorted(&days));

            for day in days {
                assert!(*day.date() >= first);
                assert!(*day.date() <= last);
            }
        }
    }
}
