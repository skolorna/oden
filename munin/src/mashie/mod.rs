mod scrape;

use std::ops::RangeInclusive;

pub use scrape::*;

use reqwest::{header::CONTENT_LENGTH, Client};
use select::{document::Document, predicate::Name};
use serde::Deserialize;
use stor::menu::{Patch, Supplier};
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
            "{host}/public/app/internal/execute-query?country=se"
        ))
        .header(CONTENT_LENGTH, "0")
        .send()
        .await?;

    let menus = res.json::<Vec<Menu>>().await?;

    Ok(menus)
}

#[instrument(level = "debug", skip(client))]
pub async fn query_menu(client: &Client, host: &str, menu_slug: &str) -> Result<Menu> {
    let menus = list_menus(client, host).await?;
    let menu = menus
        .into_iter()
        .find(|m| m.id == menu_slug)
        .ok_or(Error::MenuNotFound)?;

    Ok(menu)
}

#[instrument(level = "debug", skip_all, fields(host, reference, ?dates))]
pub async fn list_days(
    client: &Client,
    host: &str,
    reference: &str,
    dates: RangeInclusive<Date>,
) -> Result<ListDays> {
    let menu = query_menu(client, host, reference).await?;
    let url = format!("{}/{}", host, menu.path);
    let html = reqwest::get(&url).await?.text().await?;
    let doc = Document::from(html.as_str());
    let days = scrape_days(&doc)
        .filter(|day| dates.contains(&day.date))
        .collect();

    let title = doc.find(Name("title")).next().map(|t| t.text());
    let title = title.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());

    Ok(ListDays {
        menu: Patch {
            title,
            ..Default::default()
        },
        days,
    })
}

/// Automagically generate a Mashie client.
macro_rules! mashie_impl {
    ($host:literal, $supplier:expr) => {
        use std::ops::RangeInclusive;

        use reqwest::Client;
        use stor::Menu;
        use time::Date;
        use $crate::{mashie, supplier::ListDays, Result};

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
            reference: &str,
            dates: RangeInclusive<Date>,
        ) -> Result<ListDays> {
            mashie::list_days(client, HOST, reference, dates).await
        }

        #[cfg(test)]
        mod auto_tests {
            use reqwest::Client;

            #[tokio::test]
            async fn nonempty() {
                let menus = super::list_menus(&Client::new()).await.unwrap();
                assert!(!menus.is_empty());
            }
        }
    };
}

pub(crate) use mashie_impl;

use crate::{supplier::ListDays, Error, Result};

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
