use std::str::FromStr;

use chrono::{Datelike, Local, NaiveDate};
use select::{
    document::Document,
    node::Node,
    predicate::{Class, Predicate},
};

use crate::{menus::meal::Meal, types::day::Day};

/// Parse a month literal in Swedish. Returns the month, starting from 1 with January.
/// ```
/// use munin_lib::menus::mashie::scrape::parse_month;
///
/// assert_eq!(parse_month("jun"), Some(6));
/// assert!(parse_month("may").is_none()); // maj is correct
/// assert!(parse_month("Jan").is_none()); // case sensitive
/// ```
#[must_use]
pub fn parse_month(m: &str) -> Option<u32> {
    match m {
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
    }
}

fn parse_date_literal(literal: &str) -> Option<NaiveDate> {
    let mut segments = literal.split_whitespace();

    let d = segments.next()?.parse::<u32>().ok()?;
    let m = parse_month(segments.next()?)?;

    // Accept None as year, but not Some(&str) that doesn't parse to i32.
    let y = segments
        .next()
        .map_or_else(|| Ok(Local::now().year()), str::parse)
        .ok()?;

    NaiveDate::from_ymd_opt(y, m, d)
}

fn parse_day_node(node: &Node) -> Option<Day> {
    let date_literal = node
        .find(Class("panel-heading").descendant(Class("pull-right")))
        .next()?
        .text();
    let date = parse_date_literal(&date_literal)?;

    let meals = node
        .find(Class("app-daymenu-name"))
        .filter_map(|n| Meal::from_str(&n.text()).ok())
        .collect::<Vec<Meal>>();

    Day::new_opt(date, meals)
}

#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn scrape_mashie_days(doc: &Document) -> Vec<Day> {
    let day_elems = doc.find(Class("panel-group").child(Class("panel")));
    day_elems
        .filter_map(|n| parse_day_node(&n))
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Local};
    use std::convert::TryFrom;

    use crate::{menus::mashie::query_menu, util::is_sorted};

    use super::*;

    #[test]
    fn parsing_months() {
        let months = vec![
            "jan", "feb", "mar", "apr", "maj", "jun", "jul", "aug", "sep", "okt", "nov", "dec",
        ];

        for (i, month) in months.into_iter().enumerate() {
            let n = u32::try_from(i + 1).unwrap();
            assert_eq!(parse_month(month), Some(n));
        }

        assert!(parse_month("jAN").is_none());
        assert!(parse_month("juni").is_none());
    }

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

    #[tokio::test]
    async fn scrape_days_test() {
        let host = "https://sodexo.mashie.com";
        let menu = query_menu(host, "4854efa1-29b3-4534-8820-abeb008ed759")
            .await
            .unwrap();
        assert_eq!(menu.title, "Karolina, Pysslingen");

        let url = format!("{}/{}", host, menu.path);
        let html = reqwest::get(&url).await.unwrap().text().await.unwrap();
        let doc = Document::from(html.as_str());
        let days = scrape_mashie_days(&doc);

        assert!(!days.is_empty());
        assert!(is_sorted(&days));

        for day in days {
            assert!(!day.meals().is_empty());
        }

        assert!(scrape_mashie_days(&Document::from("<h1>no days</h1>")).is_empty());
    }
}
