use select::{
    document::Document,
    node::Node,
    predicate::{Class, Predicate},
};
use stor::{Day, Meal};
use time::{Date, Month, OffsetDateTime};
use time_tz::OffsetDateTimeExt;
use tracing::{error, instrument};

/// Parse a month literal in Swedish.
#[must_use]
#[instrument]
pub fn parse_month(m: &str) -> Option<Month> {
    match m {
        "jan" => Some(Month::January),
        "feb" => Some(Month::February),
        "mar" => Some(Month::March),
        "apr" => Some(Month::April),
        "maj" => Some(Month::May),
        "jun" => Some(Month::June),
        "jul" => Some(Month::July),
        "aug" => Some(Month::August),
        "sep" => Some(Month::September),
        "okt" => Some(Month::October),
        "nov" => Some(Month::November),
        "dec" => Some(Month::December),
        _ => {
            error!("unable to parse invalid month");
            None
        }
    }
}

#[instrument]
fn parse_date_literal(literal: &str) -> Option<Date> {
    let mut segments = literal.split_whitespace();

    let d = segments.next()?.parse().ok()?;
    let m = parse_month(segments.next()?)?;

    // Accept None as year, but not Some(&str) that doesn't parse to i32.
    let y = segments
        .next()
        .map_or_else(|| Ok(OffsetDateTime::now_utc().to_timezone(crate::TZ).year()), str::parse)
        .ok()?;

    Date::from_calendar_date(y, m, d).ok()
}

fn parse_day_node(node: &Node) -> Option<Day> {
    let date_literal = node
        .find(Class("panel-heading").descendant(Class("pull-right")))
        .next()?
        .text();
    let date = parse_date_literal(&date_literal)?;

    let meals = node
        .find(Class("app-daymenu-name"))
        .filter_map(|n| n.text().parse().ok())
        .collect::<Vec<Meal>>();

    Day::new(date, meals)
}

#[allow(clippy::module_name_repetitions)]
pub fn scrape_days(doc: &Document) -> impl Iterator<Item = Day> + '_ {
    let day_elems = doc.find(Class("panel-group").child(Class("panel")));
    day_elems.filter_map(|n| parse_day_node(&n))
}

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use time::macros::date;

    use crate::{mashie::query_menu};

    use super::*;

    #[test]
    fn parse_dates_test() {
        let year = OffsetDateTime::now_utc().year();

        assert_eq!(
            parse_date_literal("05 jun").unwrap(),
            Date::from_calendar_date(year, Month::June, 5).unwrap()
        );
        assert_eq!(
            parse_date_literal("17 maj 2020").unwrap(),
            date!(2020-05-17)
        );
        assert_eq!(
            parse_date_literal("29 feb 2020").unwrap(),
            date!(2020-02-29)
        );

        assert!(parse_date_literal("May 17").is_none());
        assert!(parse_date_literal("2020-05-17T00:00:00.000+02:00").is_none());
        assert!(parse_date_literal("17 maj INVALIDYEAR").is_none());
        assert!(parse_date_literal("29 feb 2021").is_none());
    }

    #[tokio::test]
    async fn scrape_days_test() {
        let host = "https://sodexo.mashie.com";
        let menu = query_menu(&Client::new(), host, "4854efa1-29b3-4534-8820-abeb008ed759")
            .await
            .unwrap();
        assert_eq!(menu.title, "Karolina, Pysslingen");

        let url = format!("{}/{}", host, menu.path);
        let html = reqwest::get(&url).await.unwrap().text().await.unwrap();
        let doc = Document::from(html.as_str());
        let days = scrape_days(&doc).collect::<Vec<_>>();
        
        assert!(!days.is_empty());

        assert_eq!(scrape_days(&Document::from("<h1>no days</h1>")).count(), 0);
    }
}
