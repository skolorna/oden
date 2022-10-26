use std::str::FromStr;

use chrono::{DateTime, Datelike, NaiveDate};
use reqwest::{header, Client, StatusCode};
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use tracing::{error, instrument};

use crate::errors::{Error, Result};
use crate::menus::supplier::Supplier;
use crate::util::{extract_digits, last_path_segment, parse_weekday};
use crate::{Day, Meal, Menu, MenuSlug};

#[instrument(err)]
pub async fn list_menus() -> Result<Vec<Menu>> {
    let html = reqwest::get("https://www.sabis.se/restauranger-cafeer/vara-foretagsrestauranger/")
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

            Some(Menu::new(MenuSlug::new(Supplier::Sabis, local_id?), title))
        })
        .collect::<Vec<_>>();

    Ok(menus)
}

#[instrument(fields(%first, %last))]
pub async fn list_days(menu_slug: &str, first: NaiveDate, last: NaiveDate) -> Result<Vec<Day>> {
    let url = format!(
        "https://www.sabis.se/{}/dagens-lunch/",
        urlencoding::encode(menu_slug)
    );
    let client = Client::new();
    let res = client.get(&url).send().await?;
    let http_date = res.headers().get(header::DATE).unwrap().to_str().unwrap(); // the date header is always present
    let res_timestamp = DateTime::parse_from_rfc2822(http_date).unwrap();

    if res.status() == StatusCode::NOT_FOUND {
        return Err(Error::MenuNotFound);
    }

    let html = res.text().await?;
    let doc = Document::from(html.as_str());

    let entry_title = doc
        .find(Class("menu-block__title"))
        .next()
        .map(|el| el.text())
        .ok_or_else(|| {
            error!("no title found for Sabis menu \"{}\"!", menu_slug);
            Error::ScrapeError { context: html }
        })?;

    let week_num = extract_digits(entry_title.chars(), 10);

    let days = doc
        .find(Class("menu-block__days").child(Name("li")))
        .filter_map(|n| {
            let weekday_literal = n.find(Class("menu-block__day-title")).next()?.text();
            let weekday = parse_weekday(weekday_literal.as_str())?;
            let date = NaiveDate::from_isoywd_opt(res_timestamp.year(), week_num, weekday)?;

            if date < first || date > last {
                return None;
            }

            let meals = n
                .find(Class("menu-block__dish"))
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

        assert!(menus.len() >= 10);
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
