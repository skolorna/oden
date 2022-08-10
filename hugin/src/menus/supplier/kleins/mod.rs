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
    menus::{mashie::scrape::scrape_mashie_days, supplier::Supplier},
    util::last_path_segment,
    Day, Menu, MenuSlug,
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
async fn raw_list_schools() -> Result<Vec<KleinsSchool>> {
    let client = Client::new();
    let html = fetch(&client, "https://www.kleinskitchen.se/skolor/")
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

#[instrument(err)]
pub async fn list_menus() -> Result<Vec<Menu>> {
    let schools = raw_list_schools().await?;

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

async fn raw_query_school(school_slug: &str) -> Result<QuerySchoolResponse> {
    let client = Client::new();
    let url = format!(
        "https://www.kleinskitchen.se/skolor/{}",
        urlencoding::encode(school_slug)
    );
    let html = fetch(&client, &url).await?.text().await?;
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
pub async fn list_days(menu_slug: &str, first: NaiveDate, last: NaiveDate) -> Result<Vec<Day>> {
    let menu_url = {
        let res = raw_query_school(menu_slug).await?;
        res.menu_url
    };
    let html = reqwest::get(&menu_url).await?.text().await?;
    let doc = Document::from(html.as_str());
    let days = scrape_mashie_days(&doc)
        .into_iter()
        .filter(|day| day.is_between(first, last))
        .collect();

    Ok(days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn kleins_list_schools_raw() {
        let schools = raw_list_schools().await.unwrap();

        assert!(schools.len() > 50);
    }

    #[tokio::test]
    async fn kleins_query_school_raw() {
        let res = raw_query_school("viktor-rydberg-grundskola-jarlaplan")
            .await
            .unwrap();

        assert_eq!(
            res.menu_url,
            "https://mpi.mashie.com/public/app/KK%20VRVasastan/4ad9e398"
        );

        assert!(
            raw_query_school("viktor-rydberg-grundskola-jarlaplan?a=evil")
                .await
                .is_err()
        );
        assert!(raw_query_school("nonexistent").await.is_err());
    }

    #[tokio::test]
    async fn kleins_list_days() {
        let days = list_days(
            "forskolan-pingvinen",
            NaiveDate::from_ymd(1970, 1, 1),
            NaiveDate::from_ymd(2077, 1, 1),
        )
        .await
        .unwrap();

        assert!(!days.is_empty());
    }
}
