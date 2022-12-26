mod scrape;

pub use scrape::*;

use reqwest::{header::CONTENT_LENGTH, Client};
use select::document::Document;
use serde::Deserialize;
use stor::menu::{Supplier};
use time::Date;
use tracing::instrument;

#[derive(Deserialize, Debug)]
pub struct Menu {
    id: String,
    title: String,
    #[serde(rename(deserialize = "url"))]
    path: String,
}

impl Menu {
    #[must_use]
    pub fn normalize(self, as_supplier: Supplier) -> stor::Menu {
        stor::Menu::from_supplier(as_supplier, self.id, self.title)
    }
}

#[instrument(err, skip(client))]
pub async fn list_menus(client: &Client, host: &str) -> Result<Vec<Menu>> {
    let res = client
        .post(&format!(
            "{}/public/app/internal/execute-query?country=se",
            host
        ))
        .header(CONTENT_LENGTH, "0")
        .send()
        .await?;

    let menus = res.json::<Vec<Menu>>().await?;

    Ok(menus)
}

#[instrument(err, skip(client))]
pub async fn query_menu(client: &Client, host: &str, menu_slug: &str) -> Result<Menu> {
    let menus = list_menus(client, host).await?;
    let menu = menus
        .into_iter()
        .find(|m| m.id == menu_slug)
        .ok_or(Error::MenuNotFound)?;

    Ok(menu)
}

#[instrument(fields(%first, %last))]
pub async fn list_days(
    client: &Client,
    host: &str,
    menu_slug: &str,
    first: Date,
    last: Date,
) -> Result<Vec<stor::Day>> {
    let menu = query_menu(client, host, menu_slug).await?;
    let url = format!("{}/{}", host, menu.path);
    let html = reqwest::get(&url).await?.text().await?;
    let doc = Document::from(html.as_str());
    let days = scrape_days(&doc)
        .into_iter()
        .filter(|day| (first..=last).contains(&day.date()))
        .collect();

    // debug_assert!(is_sorted(&days));

    Ok(days)
}

/// Automagically generate a Mashie client.
macro_rules! mashie_impl {
    ($host:literal, $supplier:expr) => {
        use time::Date;
        use reqwest::Client;
        use $crate::{mashie, Result};
        use stor::{Day, Menu};

        const HOST: &str = $host;

        pub async fn list_menus(client: &Client) -> Result<Vec<Menu>> {
            let menus = mashie::list_menus(client, HOST)
                .await?
                .into_iter()
                .map(|m| m.normalize($supplier))
                .collect();

            Ok(menus)
        }

        pub async fn list_days(
            client: &Client,
            menu_slug: &str,
            first: Date,
            last: Date,
        ) -> Result<Vec<Day>> {
            mashie::list_days(client, HOST, menu_slug, first, last).await
        }

        #[cfg(test)]
        mod auto_tests {
            use reqwest::Client;

            #[tokio::test]
            async fn nonempty() {
                let menus = super::list_menus(&Client::new()).await.unwrap();
                println!("{:?}", &menus);
                assert!(!menus.is_empty());
            }
        }
    };
}

pub(crate) use mashie_impl;

use crate::{Result, Error};

#[cfg(test)]
mod tests {
    use reqwest::Client;

    #[tokio::test]
    async fn list_menus() {
        let menus = super::list_menus(&Client::new(), "https://sodexo.mashie.com")
            .await
            .unwrap();

        assert!(menus.len() > 100);
    }

    #[tokio::test]
    async fn query_menu() {
        let menu = super::query_menu(
            &Client::new(),
            "https://sodexo.mashie.com",
            "e8851c61-013b-4617-93d9-adab00820bcd",
        )
        .await
        .unwrap();

        assert_eq!(menu.title, "Södermalmsskolan, Södermalmsskolan");
        assert_eq!(menu.id, "e8851c61-013b-4617-93d9-adab00820bcd");

        assert!(
            super::query_menu(&Client::new(), "https://sodexo.mashie.com", "invalid")
                .await
                .is_err()
        );
    }
}
