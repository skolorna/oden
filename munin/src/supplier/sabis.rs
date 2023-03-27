use reqwest::{
    header::{self, HeaderMap},
    Client,
};
use select::{
    document::Document,
    predicate::{Class, Name},
};
use stor::Day;
use time::{Date, OffsetDateTime};
use time_tz::OffsetDateTimeExt;

use crate::{util::parse_weekday, Error, Result};

use super::ListDays;

pub const TZ: &time_tz::Tz = time_tz::timezones::db::europe::STOCKHOLM;

fn http_date(headers: &HeaderMap) -> Option<OffsetDateTime> {
    let s = headers.get(header::DATE)?.to_str().ok()?;
    httpdate::parse_http_date(s).ok().map(Into::into)
}

fn weeknum(doc: &Document) -> Option<u8> {
    let title = doc.find(Class("menu-block__title")).next()?.text();
    title.split_whitespace().rev().next()?.parse().ok()
}

pub async fn list_days(client: &Client, restaurant: &str) -> Result<ListDays> {
    let url =
        format!("https://www.sabis.se/restauranger-cafeer/vara-foretagsrestauranger/{restaurant}/");
    let res = client.get(url).send().await?;
    let date = http_date(res.headers())
        .unwrap_or_else(OffsetDateTime::now_utc)
        .to_timezone(TZ);

    let html = res.text().await?;
    let doc = Document::from(html.as_str());

    let week = weeknum(&doc).ok_or_else(|| Error::scrape_error("failed to extract week number"))?;

    let days: Vec<Day> = doc
        .find(Class("menu-block__dishes"))
        .filter_map(|n| {
            let weekday = parse_weekday(
                &n.parent()?
                    .find(Class("menu-block__day-title"))
                    .next()?
                    .text(),
            )?;
            let date = Date::from_iso_week_date(date.year(), week, weekday).ok()?;

            Some(Day {
                date,
                meals: n
                    .find(Name("li"))
                    .map(|n| n.text().trim().to_owned())
                    .collect(),
            })
        })
        .collect();

    Ok(ListDays {
        menu: Default::default(),
        days,
    })
}

#[cfg(test)]
mod tests {
    use reqwest::Client;

    #[tokio::test]
    async fn carnegie() {
        let res = super::list_days(&Client::new(), "carnegie").await.unwrap();

        assert_eq!(res.days.len(), 5);
    }
}
