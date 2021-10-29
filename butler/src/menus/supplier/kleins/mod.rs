mod fetch;

use chrono::NaiveDate;
use lazy_static::lazy_static;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use url::Url;

use crate::{
    errors::{ButlerError, ButlerResult},
    menus::{day::Day, id::MenuId, mashie::scrape::scrape_mashie_days, supplier::Supplier, Menu},
    util::last_path_segment,
};
use fetch::fetch;

lazy_static! {
    static ref S_SCHOOL_NAME: Selector = Selector::parse(".school .school-title a").unwrap();
    static ref S_PAGE_TITLE: Selector = Selector::parse("h1.page-title").unwrap();
    static ref S_MENU_IFRAME: Selector = Selector::parse("iframe").unwrap();
}

#[derive(Debug)]
struct KleinsSchool {
    title: String,
    slug: String,
}

impl KleinsSchool {
    pub fn into_menu(self) -> Menu {
        let id = MenuId::new(Supplier::Kleins, self.slug);

        Menu::new(id, self.title)
    }
}

async fn raw_list_schools() -> ButlerResult<Vec<KleinsSchool>> {
    let client = Client::new();
    let html = fetch(&client, "https://www.kleinskitchen.se/skolor/")
        .await?
        .text()
        .await?;
    let doc = Html::parse_document(&html);
    let schools = doc
        .select(&S_SCHOOL_NAME)
        .filter_map(|elem| {
            let title = elem.text().next()?.trim().to_owned();
            let url = Url::parse(elem.value().attr("href")?).ok()?;
            let slug = last_path_segment(&url)?.to_owned();

            Some(KleinsSchool { title, slug })
        })
        .collect();

    Ok(schools)
}

pub async fn list_menus() -> ButlerResult<Vec<Menu>> {
    let schools = raw_list_schools().await?;

    let menus = schools.into_iter().map(|s| s.into_menu()).collect();

    Ok(menus)
}

#[derive(Debug)]
struct QuerySchoolResponse {
    school: KleinsSchool,
    menu_url: String,
}

fn extract_menu_url(iframe_elem: ElementRef) -> Option<String> {
    let iframe_src = iframe_elem.value().attr("src")?;
    let menu_url = iframe_src.replace("/menu/", "/app/");

    Some(menu_url)
}

async fn raw_query_school(school_slug: &str) -> ButlerResult<QuerySchoolResponse> {
    let client = Client::new();
    let url = format!(
        "https://www.kleinskitchen.se/skolor/{}",
        urlencoding::encode(school_slug)
    );
    let html = fetch(&client, &url).await?.text().await?;
    let doc = Html::parse_document(&html);

    let title = doc
        .select(&S_PAGE_TITLE)
        .next()
        .map(|elem| elem.text().next())
        .flatten()
        .ok_or(ButlerError::ScrapeError)?;
    let school = KleinsSchool {
        slug: school_slug.to_owned(),
        title: title.to_owned(),
    };

    let menu_url = doc
        .select(&S_MENU_IFRAME)
        .next()
        .map(extract_menu_url)
        .flatten()
        .ok_or(ButlerError::ScrapeError)?;

    Ok(QuerySchoolResponse { school, menu_url })
}

pub async fn query_menu(menu_id: &str) -> ButlerResult<Menu> {
    let res = raw_query_school(menu_id).await?;

    Ok(res.school.into_menu())
}

pub async fn list_days(menu_id: &str, first: NaiveDate, last: NaiveDate) -> ButlerResult<Vec<Day>> {
    let menu_url = {
        let res = raw_query_school(menu_id).await?;
        res.menu_url
    };
    let html = reqwest::get(&menu_url).await?.text().await?;
    let doc = Html::parse_document(&html);
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

        assert_eq!(res.school.title, "Viktor Rydberg Gymnasium Jarlaplan");
        assert_eq!(res.school.slug, "viktor-rydberg-grundskola-jarlaplan");

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
