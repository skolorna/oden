use chrono::{Datelike, NaiveDate};
use futures::{
    stream::{self, StreamExt},
    TryFutureExt,
};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tracing::{error, instrument};

use crate::{dedup_day_dates, errors::Result, Error, Menu, MenuSlug};

use super::Supplier;

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
    fn to_menu(&self, district_name: &str) -> Option<Menu> {
        if self.name.to_lowercase().contains("info") {
            None
        } else {
            Some(Menu::new(
                MenuSlug::new(Supplier::Skolmaten, self.id.to_string()),
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
pub async fn list_menus(client: &Client) -> Result<Vec<Menu>> {
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
    fn normalize(self) -> Option<crate::Meal> {
        self.value.parse().ok()
    }
}

#[derive(Deserialize, Debug)]
struct Day {
    year: i32,
    month: u32,
    day: u32,
    meals: Option<Vec<Meal>>,
}

impl Day {
    /// Maps `NaiveDate::from_ymd_opt` and creates a [`Day`]; thus, `None`
    /// is returned on invalid dates such as *February 29, 2021*. Also,
    /// `None` is returned if `meals` is `None`.
    fn normalize(self) -> Option<crate::Day> {
        let date = NaiveDate::from_ymd_opt(self.year, self.month, self.day)?;
        let meals: Vec<crate::Meal> = self
            .meals?
            .into_iter()
            .filter_map(Meal::normalize)
            .collect();

        crate::Day::new_opt(date, meals)
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
pub(super) struct SkolmatenMenu {
    // is_feedback_allowed: bool,
    weeks: Vec<Week>,
    // station: DetailedStation,
    // id: u64,
    // bulletins: Vec<Bulletin>,
}

#[derive(Deserialize, Debug)]
pub(super) struct SkolmatenMenuResponse {
    menu: SkolmatenMenu,
}

impl SkolmatenMenuResponse {
    pub(crate) fn into_days_iter(self) -> impl Iterator<Item = crate::Day> {
        self.menu
            .weeks
            .into_iter()
            .flat_map(|week| week.days)
            .filter_map(Day::normalize)
    }
}

#[derive(PartialEq, Debug)]
struct SkolmatenWeekSpan {
    year: i32,
    week_of_year: u32,
    count: u8,
}

#[instrument(skip(client))]
async fn raw_fetch_menu(
    client: &Client,
    station_id: u64,
    span: &SkolmatenWeekSpan,
) -> Result<SkolmatenMenuResponse> {
    let path = format!(
        "menu?station={}&year={}&weekOfYear={}&count={}",
        station_id, span.year, span.week_of_year, span.count
    );

    let res = fetch(client, &path).await?;
    let status = res.status();
    match res.json::<SkolmatenMenuResponse>().await {
        Ok(res) => Ok(res),
        Err(e) => {
            if status != StatusCode::NOT_FOUND {
                error!("{}", e);
            }

            Err(Error::MenuNotFound)
        }
    }
}

/// Generate a series of queries because the Skolmaten API cannot handle more than one year per request.
fn generate_week_spans(first: NaiveDate, last: NaiveDate) -> Vec<SkolmatenWeekSpan> {
    assert!(first <= last, "First must not be after last.");

    let mut spans: Vec<SkolmatenWeekSpan> = Vec::new();
    let mut segment_start = first;

    while last > segment_start {
        use std::convert::TryFrom;

        let year = segment_start.year();

        let segment_end = if last.year() == year {
            last
        } else {
            NaiveDate::from_ymd(year, 12, 31)
        };

        let diff = segment_end - segment_start;
        let weeks = u8::try_from((diff.num_days() + 6) / 7).unwrap(); // Round the number of weeks up, not down.

        let mut week_of_year = segment_start.iso_week().week();

        // skolmaten.se cannot handle week numbers above 52 for some reason.
        if week_of_year > 52 {
            week_of_year = 1;
        }

        spans.push(SkolmatenWeekSpan {
            year,
            week_of_year,
            count: weeks,
        });

        segment_start = NaiveDate::from_yo(year + 1, 1);
    }

    spans
}

/// List days of a particular Skolmaten menu.
#[instrument(skip(client), fields(%first, %last))]
pub async fn list_days(
    client: &Client,
    station: u64,
    first: NaiveDate,
    last: NaiveDate,
) -> Result<Vec<crate::Day>> {
    let spans = generate_week_spans(first, last);

    let results = stream::iter(spans)
        .map(|span| {
            let client = &client;
            async move {
                raw_fetch_menu(client, station, &span)
                    .await
                    .map(SkolmatenMenuResponse::into_days_iter)
            }
        })
        .buffer_unordered(4)
        .collect::<Vec<_>>()
        .await;

    let days_2d = results.into_iter().collect::<Result<Vec<_>>>()?;

    let mut days = days_2d
        .into_iter()
        .flatten()
        .filter(|day| day.is_between(first, last))
        .collect::<Vec<_>>();

    days.sort();
    dedup_day_dates(&mut days);

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
    use chrono::Duration;

    use super::*;

    #[tokio::test]
    async fn list_menus_test() {
        let menus = list_menus(&Client::new()).await.unwrap();

        assert!(menus.len() > 5000);

        for menu in menus {
            assert!(!menu.title().to_lowercase().contains("info"));
        }
    }

    #[tokio::test]
    async fn list_days_test() {
        let first_day = chrono::offset::Local::now().date().naive_local();
        let last_day = first_day + Duration::weeks(2);

        let days = list_days(&Client::new(), 4889403990212608, first_day, last_day)
            .await
            .unwrap();

        assert!(!days.is_empty());
    }

    #[test]
    fn week_spans() {
        let week_53 = super::generate_week_spans(
            NaiveDate::from_ymd(2020, 8, 1),
            NaiveDate::from_ymd(2021, 3, 1),
        );
        assert_eq!(
            week_53,
            [
                SkolmatenWeekSpan {
                    year: 2020,
                    week_of_year: 31,
                    count: 22,
                },
                SkolmatenWeekSpan {
                    year: 2021,
                    week_of_year: 1, // Skolmaten.se doesn't care that 53 is the correct week number.
                    count: 9,
                }
            ]
        );
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
            .meals()
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
