use std::{iter, collections::HashMap};

use futures::{
    stream::{self, StreamExt},
    TryFutureExt,
};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use stor::menu::Supplier;
use time::{Date, Month};
use tracing::{error, instrument};

use crate::{Error, Result};

/// Maximum number of concurrent HTTP requests when crawling. For comparison,
/// Firefox allows 7 concurrent requests. There is virtually no improvement for
/// values above 64, and 32 is just marginally slower (and half the memory
/// usage).
const CONCURRENT_REQUESTS: usize = 32;

#[derive(Deserialize, Debug)]
struct Province {
    id: u64,
    // name: String,
}

#[derive(Deserialize, Debug)]
struct ProvincesResponse {
    provinces: Vec<Province>,
}

#[derive(Deserialize, Debug, Clone)]
struct District {
    id: u64,
    name: String,
}

#[derive(Deserialize, Debug)]
struct DistrictsResponse {
    districts: Vec<District>,
}

#[derive(Deserialize, Debug, Clone)]
struct Station {
    id: u64,
    name: String,
}

#[derive(Deserialize, Debug)]
struct StationsResponse {
    stations: Vec<Station>,
}

impl Station {
    fn to_menu(&self, district_name: &str) -> Option<stor::Menu> {
        if self.name.to_lowercase().contains("info") {
            None
        } else {
            Some(stor::Menu::from_supplier(
                Supplier::Skolmaten,
                self.id.to_string(),
                format!("{}, {}", self.name.trim(), district_name),
            ))
        }
    }
}

#[instrument(skip(client))]
async fn list_provinces(client: &Client) -> Result<Vec<Province>> {
    let res = fetch(client, "provinces")
        .await?
        .json::<ProvincesResponse>()
        .await?;

    Ok(res.provinces)
}

#[instrument(skip(client))]
async fn list_districts_in_province(client: &Client, province_id: u64) -> Result<Vec<District>> {
    let res = fetch(client, &format!("districts?province={}", province_id))
        .await?
        .json::<DistrictsResponse>()
        .await?;

    Ok(res.districts)
}

#[instrument(skip(client))]
async fn list_stations_in_district(client: &Client, district_id: u64) -> Result<Vec<Station>> {
    let res = fetch(client, &format!("stations?district={}", district_id))
        .await?
        .json::<StationsResponse>()
        .await?;

    Ok(res.stations)
}

#[instrument(err, skip(client))]
pub async fn list_menus(client: &Client) -> Result<Vec<stor::Menu>> {
    let provinces = list_provinces(client).await?;

    let mut districts = Vec::new();

    let mut districts_stream = stream::iter(provinces)
        .map(|province| list_districts_in_province(client, province.id))
        .buffer_unordered(CONCURRENT_REQUESTS);

    while let Some(res) = districts_stream.next().await {
        districts.extend(res?);
    }

    let mut menus = Vec::new();

    let mut menus_stream = stream::iter(districts)
        .map(|district| {
            list_stations_in_district(client, district.id)
                .map_ok(|stations| (district.name, stations))
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    while let Some(res) = menus_stream.next().await {
        let (district_name, stations) = res?;
        menus.extend(
            stations
                .into_iter()
                .filter_map(|s| s.to_menu(&district_name)),
        );
    }

    Ok(menus)
}

#[derive(Deserialize, Debug, Clone)]
struct Meal {
    value: String,
}

impl Meal {
    fn normalize(self) -> Option<stor::Meal> {
        self.value.parse().ok()
    }
}

#[derive(Deserialize, Debug)]
struct Day {
    year: i32,
    month: u8,
    day: u8,
    meals: Option<Vec<Meal>>,
}

impl Day {
    fn normalize(self) -> Option<stor::Day> {
        let date = Date::from_calendar_date(
            self.year,
            match self.month {
                1 => Month::January,
                2 => Month::February,
                3 => Month::March,
                4 => Month::April,
                5 => Month::May,
                6 => Month::June,
                7 => Month::July,
                8 => Month::August,
                9 => Month::September,
                10 => Month::October,
                11 => Month::November,
                12 => Month::December,
                _ => return None,
            },
            self.day,
        )
        .ok()?;
        let meals: Vec<stor::Meal> = self
            .meals?
            .into_iter()
            .filter_map(Meal::normalize)
            .collect();

        stor::Day::new(date, meals)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Week {
    // year: i32,
    // week_of_year: u32,
    days: Vec<Day>,
}

// #[derive(Deserialize, Debug)]
// pub(super) struct Bulletin {
//     // text: String,
// }

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Menu {
    // is_feedback_allowed: bool,
    weeks: Vec<Week>,
    // station: DetailedStation,
    // id: u64,
    // bulletins: Vec<Bulletin>,
}

#[derive(Deserialize, Debug)]
struct MenuResponse {
    menu: Menu,
}

impl MenuResponse {
    fn into_days_iter(self) -> impl Iterator<Item = stor::Day> {
        self.menu
            .weeks
            .into_iter()
            .flat_map(|week| week.days)
            .filter_map(Day::normalize)
    }
}

#[derive(PartialEq, Debug)]
struct WeekSpan {
    year: i32,
    week_of_year: u8,
    count: u8,
}

#[instrument(skip(client))]
async fn fetch_menu(client: &Client, station_id: u64, span: &WeekSpan) -> Result<MenuResponse> {
    let path = format!(
        "menu?station={}&year={}&weekOfYear={}&count={}",
        station_id, span.year, span.week_of_year, span.count
    );

    let res = fetch(client, &path).await?;
    let status = res.status();
    Ok(res.json::<MenuResponse>().await.map_err(|e| {
        if status != StatusCode::NOT_FOUND {
            error!("{}", e);
        }

        Error::MenuNotFound
    })?)
}

/// Generate a series of queries because the Skolmaten API cannot handle
/// more than one year per request.
fn week_spans(first: Date, last: Date) -> impl Iterator<Item = WeekSpan> {
    // in release mode, the iterator will yield None immediately
    debug_assert!(first <= last, "first must not be after last");

    let mut span_start = first;

    iter::from_fn(move || {
        if last <= span_start {
            return None;
        }

        let year = span_start.year();

        let span_end = if last.year() == year {
            last
        } else {
            Date::from_calendar_date(year, Month::December, 31).unwrap()
        };

        let diff = span_end - span_start;
        let weeks = u8::try_from((diff.whole_days() + 6) / 7).unwrap(); // round the number of weeks up

        let mut week_of_year = span_start.iso_week();

        // skolmaten.se cannot handle week numbers above 52
        if week_of_year > 52 {
            week_of_year = 1;
        }

        span_start = Date::from_ordinal_date(year + 1, 1).unwrap();

        Some(WeekSpan {
            year,
            week_of_year,
            count: weeks,
        })
    })
}

/// List days of a particular Skolmaten menu.
#[instrument(skip(client), fields(%first, %last))]
pub async fn list_days(
    client: &Client,
    station: u64,
    first: Date,
    last: Date,
) -> Result<Vec<stor::Day>> {
    let results = stream::iter(week_spans(first, last))
        .map(|span| {
            let client = &client;
            async move {
                fetch_menu(client, station, &span)
                    .await
                    .map(MenuResponse::into_days_iter)
            }
        })
        .buffer_unordered(4)
        .collect::<Vec<_>>()
        .await;

    let days_2d = results.into_iter().collect::<Result<Vec<_>>>()?;

    let mut days = days_2d
        .into_iter()
        .flatten()
        .filter(|day| (first..=last).contains(&day.date()))
        .collect::<Vec<_>>();

    days.sort();
    days.dedup_by_key(|d| *d.date());

    Ok(days)
}

#[instrument(err)]
async fn fetch(client: &Client, path: &str) -> reqwest::Result<reqwest::Response> {
    let url = format!("https://skolmaten.se/api/4/{}", path);

    client
        .get(&url)
        .header("API-Version", "4.0")
        .header("Client-Token", "web")
        .header("Client-Version-Token", "web")
        .header("Locale", "sv_SE")
        .send()
        .await
}

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use time::{macros::date, Duration, OffsetDateTime};
    use time_tz::OffsetDateTimeExt;

    use super::WeekSpan;

    #[tokio::test]
    async fn list_menus() {
        let menus = super::list_menus(&Client::new()).await.unwrap();

        assert!(menus.len() > 5000);

        for menu in menus {
            assert!(!menu.title.to_lowercase().contains("info"));
        }
    }

    #[tokio::test]
    async fn list_days() {
        let first_day = OffsetDateTime::now_utc().to_timezone(crate::TZ).date();
        let last_day = first_day + Duration::weeks(2);

        let days = super::list_days(&Client::new(), 4889403990212608, first_day, last_day)
            .await
            .unwrap();

        assert!(!days.is_empty());
    }

    #[test]
    fn week_spans() {
        let mut it = super::week_spans(date!(2020 - 08 - 01), date!(2021 - 03 - 01));
        assert_eq!(
            it.next(),
            Some(WeekSpan {
                year: 2020,
                week_of_year: 31,
                count: 22,
            }),
        );
        assert_eq!(
            it.next(),
            Some(WeekSpan {
                year: 2021,
                week_of_year: 1, // Skolmaten.se doesn't care that 53 is the correct week number.
                count: 9,
            })
        );
        assert!(it.next().is_none());
    }

    #[test]
    fn convert_day() {
        let meals = vec![super::Meal {
            value: "Fisk Björkeby".to_owned(),
        }];

        assert_eq!(
            super::Day {
                meals: Some(meals.clone()),
                year: 2020,
                month: 1,
                day: 1,
            }
            .normalize()
            .unwrap()
            .meals
            .into_inner()
            .len(),
            1
        );
        assert!(super::Day {
            meals: Some(meals),
            year: 2021,
            month: 2,
            day: 29
        }
        .normalize()
        .is_none());
    }
}
