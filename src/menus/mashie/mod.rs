pub mod scrape;

use chrono::NaiveDate;
use reqwest::{header::CONTENT_LENGTH, Client};
use scraper::Html;
use serde::Deserialize;

use crate::{
    errors::{NotFoundError, Result},
    menus::mashie::scrape::scrape_mashie_days,
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
        Menu::new(id, self.title, provider)
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
    let days = scrape_mashie_days(&doc)?
        .into_iter()
        .filter(|day| day.date >= first && day.date <= last)
        .collect();

    Ok(days)
}

#[cfg(test)]
mod tests {
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
}
