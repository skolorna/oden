pub mod scrape;

use chrono::NaiveDate;
use reqwest::{header::CONTENT_LENGTH, Client};
use scraper::Html;
use serde::Deserialize;

use crate::{
    errors::{NotFoundError, Result},
    menus::mashie::scrape::scrape_mashie_days,
    util::is_sorted,
};

use super::{day::Day, id::MenuID, provider::Provider, Menu};

#[derive(Deserialize, Debug)]
pub struct MashieMenu {
    id: String,
    title: String,
    #[serde(rename(deserialize = "url"))]
    path: String,
}

impl MashieMenu {
    pub fn into_menu(self, provider: Provider) -> Menu {
        let id = MenuID::new(provider, self.id);
        Menu::new(id, self.title)
    }
}

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

pub async fn query_menu(host: &str, menu_id: &str) -> Result<MashieMenu> {
    let menus = list_menus(host).await?;
    let menu = menus
        .into_iter()
        .find(|m| m.id == menu_id)
        .ok_or(NotFoundError::MenuNotFoundError)?;

    Ok(menu)
}

pub async fn list_days(
    host: &str,
    menu_id: &str,
    first: NaiveDate,
    last: NaiveDate,
) -> Result<Vec<Day>> {
    let menu = query_menu(host, menu_id).await?;
    let url = format!("{}/{}", host, menu.path);
    let html = reqwest::get(&url).await?.text().await?;
    let doc = Html::parse_document(&html);
    let days: Vec<Day> = scrape_mashie_days(&doc)
        .into_iter()
        .filter(|day| day.date >= first && day.date <= last)
        .collect();

    debug_assert!(is_sorted(&days));

    Ok(days)
}

macro_rules! mashie_impl {
    ($host:literal, $provider:expr) => {
        use crate::errors::Result;
        use crate::menus::{day::Day, mashie, Menu};
        use chrono::NaiveDate;

        const HOST: &str = $host;

        pub async fn list_menus() -> Result<Vec<Menu>> {
            let menus = mashie::list_menus(HOST)
                .await?
                .into_iter()
                .map(|m| m.into_menu($provider))
                .collect();

            Ok(menus)
        }

        pub async fn query_menu(menu_id: &str) -> Result<Menu> {
            let menu = mashie::query_menu(HOST, menu_id)
                .await?
                .into_menu($provider);

            Ok(menu)
        }

        pub async fn list_days(
            menu_id: &str,
            first: NaiveDate,
            last: NaiveDate,
        ) -> Result<Vec<Day>> {
            mashie::list_days(HOST, menu_id, first, last).await
        }
    };
}

#[cfg(test)]
mod tests {
    use chrono::{offset, Duration};

    use super::*;

    #[actix_rt::test]
    async fn list_menus_test() {
        let menus = list_menus("https://sodexo.mashie.com").await.unwrap();

        assert!(menus.len() > 100);
    }

    #[actix_rt::test]
    async fn query_menu_test() {
        let menu = query_menu(
            "https://sodexo.mashie.com",
            "10910e60-20ca-4478-b864-abd8007ad970",
        )
        .await
        .unwrap();

        assert_eq!(menu.title, "Södermalmsskolan");
        assert_eq!(menu.id, "10910e60-20ca-4478-b864-abd8007ad970");

        assert!(query_menu("https://sodexo.mashie.com", "invalid")
            .await
            .is_err());
    }

    mod impl_test {
        use crate::util::is_sorted;

        use super::*;

        mashie_impl!("https://sodexo.mashie.com", Provider::Sodexo);

        const MENU_ID: &str = "312dd0ae-3ebd-49d9-870e-abeb008c0e4b";

        #[actix_rt::test]
        async fn list_menus_test() {
            let menus = list_menus().await.unwrap();
            assert!(menus.len() > 100);
        }

        #[actix_rt::test]
        async fn query_menu_test() {
            let menu = query_menu(MENU_ID).await.unwrap();
            assert_eq!(menu.title, "Loket, Pysslingen");
            assert_eq!(menu.id.local_id, MENU_ID);

            assert!(query_menu("unexisting").await.is_err());
        }

        #[actix_rt::test]
        async fn list_days_test() {
            let first = offset::Utc::today().naive_utc();
            let last = first + Duration::days(365);

            let days = list_days(MENU_ID, first, last).await.unwrap();

            assert!(days.len() > 5);
            assert!(is_sorted(&days));

            for day in days {
                assert!(day.date >= first);
                assert!(day.date <= last);
            }
        }
    }
}
