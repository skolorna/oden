mod fetch;

use chrono::NaiveDate;
use reqwest::Client;
use select::{
    document::Document,
    node::Node,
    predicate::{Class, Name, Predicate},
};
use tracing::instrument;

use crate::{
    errors::{Error, Result},
    mashie,
    util::last_path_segment,
    Day, Menu, MenuSlug, Supplier,
};
use fetch::fetch;

#[derive(Debug)]
struct KleinsSchool {
    title: String,
    slug: String,
}

impl KleinsSchool {
    pub fn normalize(self) -> Menu {
        let id = MenuSlug::new(Supplier::Kleins, self.slug);

        Menu::new(id, self.title)
    }
}

#[instrument(err)]
async fn raw_list_schools(client: &Client) -> Result<Vec<KleinsSchool>> {
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

            Some(KleinsSchool { title, slug })
        })
        .collect();

    Ok(schools)
}

#[instrument(err, skip(client))]
pub async fn list_menus(client: &Client) -> Result<Vec<Menu>> {
    let schools = raw_list_schools(client).await?;

    let menus = schools.into_iter().map(KleinsSchool::normalize).collect();

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

async fn raw_query_school(client: &Client, school_slug: &str) -> Result<QuerySchoolResponse> {
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
        .ok_or_else(|| Error::ScrapeError {
            context: html.clone(),
        })?;

    Ok(QuerySchoolResponse { menu_url })
}

#[instrument(fields(%first, %last))]
pub async fn list_days(
    client: &Client,
    menu_slug: &str,
    first: NaiveDate,
    last: NaiveDate,
) -> Result<Vec<Day>> {
    let menu_url = {
        let res = raw_query_school(client, menu_slug).await?;
        res.menu_url
    };
    let html = reqwest::get(&menu_url).await?.text().await?;
    let doc = Document::from(html.as_str());
    let days = mashie::scrape_days(&doc)
        .filter(|day| day.is_between(first, last))
        .collect();

    Ok(days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn kleins_list_schools_raw() {
        let schools = raw_list_schools(&Client::new()).await.unwrap();

        assert!(schools.len() > 50);
    }

    #[tokio::test]
    async fn kleins_query_school_raw() {
        let res = raw_query_school(&Client::new(), "viktor-rydberg-grundskola-jarlaplan")
            .await
            .unwrap();

        assert_eq!(
            res.menu_url,
            "https://mpi.mashie.com/public/app/KK%20VRVasastan/4ad9e398"
        );

        assert!(
            raw_query_school(&Client::new(), "viktor-rydberg-grundskola-jarlaplan?a=evil")
                .await
                .is_err()
        );
        assert!(raw_query_school(&Client::new(), "nonexistent")
            .await
            .is_err());
    }

    #[tokio::test]
    async fn kleins_list_days() {
        let days = list_days(
            &Client::new(),
            "forskolan-pingvinen",
            NaiveDate::from_ymd(1970, 1, 1),
            NaiveDate::from_ymd(2077, 1, 1),
        )
        .await
        .unwrap();

        assert!(!days.is_empty());
    }
}
