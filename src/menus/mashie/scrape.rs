use chrono::{Datelike, Local, NaiveDate};
use lazy_static::lazy_static;
use scraper::{ElementRef, Html, Selector};

use crate::menus::{day::Day, meal::Meal};

lazy_static! {
    static ref S_DAY: Selector = Selector::parse(".panel-group > .panel").unwrap();
    static ref S_DATE: Selector = Selector::parse(".panel-heading .pull-right").unwrap();
    static ref S_MEAL: Selector = Selector::parse(".app-daymenu-name").unwrap();
}

fn parse_date_literal(literal: &str) -> Option<NaiveDate> {
    let mut segments = literal.split_whitespace();

    let d = segments.next()?.parse::<u32>().ok()?;

    let m = match segments.next()? {
        "jan" => Some(1),
        "feb" => Some(2),
        "mar" => Some(3),
        "apr" => Some(4),
        "maj" => Some(5),
        "jun" => Some(6),
        "jul" => Some(7),
        "aug" => Some(8),
        "sep" => Some(9),
        "okt" => Some(10),
        "nov" => Some(11),
        "dec" => Some(12),
        _ => None,
    }?;

    // Accept None as year, but not Some(&str) that doesn't parse to i32.
    let y = segments
        .next()
        .map(|y| y.parse::<i32>())
        .unwrap_or_else(|| Ok(Local::now().year()))
        .ok()?;

    NaiveDate::from_ymd_opt(y, m, d)
}

fn parse_meal_elem(elem: ElementRef) -> Option<Meal> {
    let text = elem.text().next()?;
    Meal::from_value(text)
}

fn parse_day_elem(elem: ElementRef) -> Option<Day> {
    let date_literal = elem
        .select(&S_DATE)
        .next()
        .map(|child| child.text().next())
        .flatten()?;
    let date = parse_date_literal(date_literal)?;

    let mut meals = elem
        .select(&S_MEAL)
        .filter_map(parse_meal_elem)
        .collect::<Vec<Meal>>();

    Day::new_opt(date, &mut meals)
}

pub fn scrape_mashie_days(doc: &Html) -> Vec<Day> {
    let day_elems = doc.select(&S_DAY);
    day_elems.filter_map(parse_day_elem).collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Local};

    use crate::{menus::mashie::query_menu, util::is_sorted};

    use super::*;

    #[test]
    fn parse_dates_test() {
        let year = Local::now().year();

        assert_eq!(
            parse_date_literal("05 jun").unwrap(),
            NaiveDate::from_ymd(year, 6, 5)
        );
        assert_eq!(
            parse_date_literal("17 maj 2020").unwrap(),
            NaiveDate::from_ymd(2020, 5, 17)
        );
        assert_eq!(
            parse_date_literal("29 feb 2020").unwrap(),
            NaiveDate::from_ymd(2020, 2, 29)
        );

        assert!(parse_date_literal("May 17").is_none());
        assert!(parse_date_literal("2020-05-17T00:00:00.000+02:00").is_none());
        assert!(parse_date_literal("17 maj INVALIDYEAR").is_none());
        assert!(parse_date_literal("29 feb 2021").is_none());
    }

    #[actix_rt::test]
    async fn scrape_days_test() {
        let host = "https://sodexo.mashie.com";
        let menu = query_menu(host, "4854efa1-29b3-4534-8820-abeb008ed759")
            .await
            .unwrap();
        assert_eq!(menu.title, "Karolina, Pysslingen");

        let url = format!("{}/{}", host, menu.path);
        let html = reqwest::get(&url).await.unwrap().text().await.unwrap();
        let doc = Html::parse_document(&html);
        let days = scrape_mashie_days(&doc);

        assert!(!days.is_empty());
        assert!(is_sorted(&days));

        for day in days {
            assert!(!day.meals.is_empty())
        }

        assert!(scrape_mashie_days(&Html::parse_fragment("<h1>no days</h1>")).is_empty());
    }
}
