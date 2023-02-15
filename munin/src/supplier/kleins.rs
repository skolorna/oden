use reqwest::{header::USER_AGENT, Client, IntoUrl, Response};
use select::{
    document::Document,
    node::Node,
    predicate::{Class, Name, Predicate},
};
use stor::menu::{Menu, Supplier};
use time::Date;
use tracing::instrument;

use crate::{mashie, util::last_path_segment, Error, Result};

use super::ListDays;

#[derive(Debug)]
struct School {
    title: String,
    slug: String,
}

impl School {
    pub fn normalize(self) -> Menu {
        Menu::from_supplier(Supplier::Kleins, self.slug, self.title)
    }
}

#[instrument(err)]
async fn list_schools(client: &Client) -> Result<Vec<School>> {
    let html = fetch(client, "https://www.kleinskitchen.se/skolor/")
        .await?
        .text()
        .await?;
    let doc = Document::from(html.as_str());
    let schools = doc
        .find(Class("school").descendant(Class("school-title").descendant(Name("a"))))
        .filter_map(|node| {
            let title = node.text().trim().to_owned();
            let slug = last_path_segment(node.attr("href")?)?.to_owned();

            Some(School { title, slug })
        })
        .collect();

    Ok(schools)
}

#[instrument(name = "kleins::list_menus", err, skip(client))]
pub async fn list_menus(client: &Client) -> Result<Vec<Menu>> {
    let schools = list_schools(client).await?;

    let menus = schools.into_iter().map(School::normalize).collect();

    Ok(menus)
}

#[derive(Debug)]
struct QuerySchoolResponse {
    // school: KleinsSchool,
    menu_url: String,
}

fn extract_menu_url(iframe_node: &Node) -> Option<String> {
    let iframe_src = iframe_node.attr("src")?;
    let menu_url = iframe_src.replace("/menu/", "/app/");

    Some(menu_url)
}

async fn query_school(client: &Client, school_slug: &str) -> Result<QuerySchoolResponse> {
    let url = format!(
        "https://www.kleinskitchen.se/skolor/{}",
        urlencoding::encode(school_slug)
    );
    let html = fetch(client, &url).await?.text().await?;
    let doc = Document::from(html.as_str());

    let menu_url = doc
        .find(Name("iframe"))
        .next()
        .and_then(|n| extract_menu_url(&n))
        .ok_or_else(|| Error::scrape_error_with_context("no iframe found", Some(&html)))?;

    Ok(QuerySchoolResponse { menu_url })
}

#[instrument(fields(%first, %last))]
pub async fn list_days(
    client: &Client,
    menu_slug: &str,
    first: Date,
    last: Date,
) -> Result<ListDays> {
    let menu_url = {
        let res = query_school(client, menu_slug).await?;
        res.menu_url
    };
    let html = reqwest::get(&menu_url).await?.text().await?;
    let doc = Document::from(html.as_str());
    let days = mashie::scrape_days(&doc)
        .filter(|day| (first..=last).contains(&day.date))
        .collect();

    Ok(ListDays { menu: None, days })
}

const UA: &str = "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:47.0) Gecko/20100101 Firefox/47.0";

async fn fetch(client: &Client, url: impl IntoUrl) -> reqwest::Result<Response> {
    client.get(url).header(USER_AGENT, UA).send().await
}

#[cfg(test)]
mod tests {
    use reqwest::{Client, StatusCode};
    use time::macros::date;

    use crate::supplier::ListDays;

    #[tokio::test]
    async fn list_schools() {
        let schools = super::list_schools(&Client::new()).await.unwrap();

        assert!(schools.len() > 50);
    }

    #[tokio::test]
    async fn query_school() {
        let res = super::query_school(&Client::new(), "viktor-rydberg-grundskola-jarlaplan")
            .await
            .unwrap();

        assert_eq!(
            res.menu_url,
            "https://mpi.mashie.com/public/app/KK%20VRVasastan/4ad9e398"
        );

        assert!(
            super::query_school(&Client::new(), "viktor-rydberg-grundskola-jarlaplan?a=evil")
                .await
                .is_err()
        );
        assert!(super::query_school(&Client::new(), "nonexistent")
            .await
            .is_err());
    }

    #[tokio::test]
    async fn list_days() {
        let ListDays { days, .. } = super::list_days(
            &Client::new(),
            "forskolan-pingvinen",
            date!(1970 - 01 - 01),
            date!(2077 - 01 - 01),
        )
        .await
        .unwrap();

        assert!(!days.is_empty());
    }

    #[tokio::test]
    async fn fetch() {
        let res = super::fetch(&Client::new(), "https://www.kleinskitchen.se/skolor/")
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }
}
