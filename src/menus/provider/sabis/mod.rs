use std::str::FromStr;

use chrono::{Datelike, NaiveDate, Utc, Weekday};
use lazy_static::lazy_static;
use reqwest::redirect::Policy;
use reqwest::{Client, StatusCode};
use scraper::{Html, Selector};
use url::Url;

use crate::errors::{Error, NotFoundError, Result};
use crate::menus::day::Day;
use crate::menus::id::MenuID;
use crate::menus::meal::Meal;
use crate::menus::provider::Provider;
use crate::menus::Menu;
use crate::util::{extract_digits, last_path_segment};

lazy_static! {
    static ref S_RESTAURANT_ANCHOR: Selector =
        Selector::parse("li.restaurant-list-item a").unwrap();
    static ref S_ENTRY_TITLE: Selector = Selector::parse(".entry-title").unwrap();
		static ref S_DAY_CONTAINER: Selector = Selector::parse(".lunch-day-container").unwrap();
		static ref S_LUNCH_DAY: Selector = Selector::parse(".lunch-day").unwrap();
		static ref S_LUNCH_DISH: Selector = Selector::parse(".lunch-dish").unwrap();
}

pub async fn list_menus() -> Result<Vec<Menu>> {
    let html = reqwest::get("https://www.sabis.se/restauranger/")
        .await?
        .text()
        .await?;
    let doc = Html::parse_document(&html);

    let menus = doc
        .select(&S_RESTAURANT_ANCHOR)
        .filter_map(|e| {
            let url = Url::parse(e.value().attr("href")?).ok()?;
            let title = e.text().collect::<_>();

            let local_id = last_path_segment(&url);

            debug_assert!(local_id.is_some());

            Some(Menu::new(
                MenuID::new(Provider::Sabis, local_id?.into()),
                title,
            ))
        })
        .collect::<Vec<_>>();

    Ok(menus)
}

pub async fn query_menu(menu_id: &str) -> Result<Menu> {
    let menus = list_menus().await?;

    menus
        .into_iter()
        .find(|m| m.id.local_id == menu_id)
        .ok_or(Error::NotFoundError(NotFoundError::MenuNotFoundError))
}

pub async fn list_days(menu_id: &str, first: NaiveDate, last: NaiveDate) -> Result<Vec<Day>> {
    let url = format!(
        "https://www.sabis.se/{}/dagens-lunch/",
        urlencoding::encode(menu_id)
    );
		let client = Client::builder().redirect(Policy::none()).build()?;
    let res = client.get(&url).send().await?;
		if res.status() == StatusCode::NOT_FOUND {
			return Err(Error::NotFoundError(NotFoundError::MenuNotFoundError));
		}
		let html = res.text().await?;
    let doc = Html::parse_document(&html);

    let chars = doc
        .select(&S_ENTRY_TITLE)
        .next()
        .ok_or(Error::InternalError)?
        .text()
        .flat_map(|s| s.chars());
		let week_num = extract_digits(chars, 10);

		let year = Utc::now().year();

		let days = doc.select(&S_DAY_CONTAINER).filter_map(|e| {
			let weekday_literal = e.select(&S_LUNCH_DAY).next()?.text().collect::<String>();
			let weekday = match weekday_literal.as_str() {
				"Måndag" => Some(Weekday::Mon),
				"Tisdag" => Some(Weekday::Tue),
				"Onsdag" => Some(Weekday::Wed),
				"Torsdag" => Some(Weekday::Thu),
				"Fredag" => Some(Weekday::Fri),
				"Lördag" => Some(Weekday::Sat),
				"Söndag" => Some(Weekday::Sun),
				_ => None,
			}?;
			let date = NaiveDate::from_isoywd_opt(year, week_num, weekday)?;

			if date < first || date > last {
				return None;
			}

			let meals = e.select(&S_LUNCH_DISH).map(|e| e.text().collect::<String>()).filter_map(|v| Meal::from_str(&v).ok()).collect::<Vec<_>>();

			Day::new_opt(date, meals)
		}).collect::<Vec<_>>();

		Ok(days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_list_menus() {
        let menus = list_menus().await.unwrap();

        dbg!(&menus);
        assert!(menus.len() > 15);
    }

    #[actix_rt::test]
    async fn test_query_menu() {
        let menu = query_menu("rosenbad").await.unwrap();

        assert_eq!(menu.title, "Restaurang Björnen");

        assert!(query_menu("om-oss").await.is_err());
    }

    #[actix_rt::test]
    async fn test_list_days() {
        let days = list_days("rosenbad", NaiveDate::from_ymd(2000, 1, 1), NaiveDate::from_ymd(2077, 1, 1)).await.unwrap();

				assert!(days.len() > 3);
				assert!(list_days("rosenbad", NaiveDate::from_ymd(2005, 1, 1), NaiveDate::from_ymd(2005, 12, 31)).await.unwrap().is_empty());
        assert!(list_days("om-oss", NaiveDate::from_ymd(2020, 1, 1), NaiveDate::from_ymd(2020, 1, 1)).await.is_err());
    }
}
