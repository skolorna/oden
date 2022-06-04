use std::str::FromStr;

use chrono::{DateTime, Datelike, NaiveDate};
use reqwest::redirect::Policy;
use reqwest::{header, Client, StatusCode};
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use tracing::error;

use crate::errors::{MuninError, MuninResult};
use crate::menus::meal::Meal;
use crate::menus::supplier::Supplier;
use crate::menus::Menu;
use crate::types::{day::Day, menu_slug::MenuSlug};
use crate::util::{extract_digits, last_path_segment, parse_weekday};

pub async fn list_menus() -> MuninResult<Vec<Menu>> {
    let html = reqwest::get("https://beta.sabis.se/restaurang-service/vara-restauranger/")
        .await?
        .text()
        .await?;
    let doc = Document::from(html.as_str());

    let menus = doc
        .find(Class("content-banner__content"))
        .filter_map(|n| {
            let title = n.find(Class("content_banner__title")).next()?.text();
            let href = n
                .find(Name("a").and(Class("content-banner__button--button")))
                .next()?
                .attr("href")?;
            let local_id = last_path_segment(href);

            debug_assert!(local_id.is_some());

            Some(Menu::new(
                MenuSlug::new(Supplier::Sabis, local_id?.into()),
                title,
            ))
        })
        .collect::<Vec<_>>();

    Ok(menus)
}

pub async fn query_menu(menu_slug: &str) -> MuninResult<Menu> {
    let menus = list_menus().await?;

    menus
        .into_iter()
        .find(|m| m.slug().local_id == menu_slug)
        .ok_or(MuninError::MenuNotFound)
}

pub async fn list_days(
    menu_slug: &str,
    first: NaiveDate,
    last: NaiveDate,
) -> MuninResult<Vec<Day>> {
    let url = format!(
        "https://www.sabis.se/{}/dagens-lunch/",
        urlencoding::encode(menu_slug)
    );
    let client = Client::builder().redirect(Policy::none()).build()?;
    let res = client.get(&url).send().await?;
    let http_date = res.headers().get(header::DATE).unwrap().to_str().unwrap(); // the date header is always present
    let res_timestamp = DateTime::parse_from_rfc2822(http_date).unwrap();

    if res.status() == StatusCode::NOT_FOUND {
        return Err(MuninError::MenuNotFound);
    }

    let html = res.text().await?;
    let doc = Document::from(html.as_str());

    let entry_title = match doc.find(Class("entry-title")).next() {
        Some(elem) => elem.text(),
        None => {
            error!("No title found for Sabis menu \"{}\"!", menu_slug);
            return Err(MuninError::ScrapeError { context: html });
        }
    };

    let week_num = extract_digits(entry_title.chars(), 10);

    let days = doc
        .find(Class("lunch-day-container"))
        .filter_map(|n| {
            let weekday_literal = n.find(Class("lunch-day")).next()?.text();
            let weekday = parse_weekday(weekday_literal.as_str())?;
            let date = NaiveDate::from_isoywd_opt(res_timestamp.year(), week_num, weekday)?;

            if date < first || date > last {
                return None;
            }

            let meals = n
                .find(Class("lunch-dish"))
                .filter_map(|n| Meal::from_str(&n.text()).ok())
                .collect::<Vec<_>>();

            Day::new_opt(date, meals)
        })
        .collect::<Vec<_>>();

    Ok(days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_menus() {
        let menus = list_menus().await.unwrap();

        assert!(menus.len() > 15);
    }

    #[tokio::test]
    async fn test_query_menu() {
        let menu = query_menu("carnegie").await.unwrap();

        assert_eq!(menu.title(), "Carnegie");

        assert!(query_menu("om-oss").await.is_err());
    }

    #[tokio::test]
    async fn test_list_days() {
        let days = list_days(
            "carnegie",
            NaiveDate::from_ymd(2000, 1, 1),
            NaiveDate::from_ymd(2077, 1, 1),
        )
        .await
        .unwrap();

        assert!(days.len() > 3);
        assert!(list_days(
            "carnegie",
            NaiveDate::from_ymd(2005, 1, 1),
            NaiveDate::from_ymd(2005, 12, 31)
        )
        .await
        .unwrap()
        .is_empty());
        assert!(list_days(
            "om-oss",
            NaiveDate::from_ymd(2020, 1, 1),
            NaiveDate::from_ymd(2020, 1, 1)
        )
        .await
        .is_err());
    }
}
