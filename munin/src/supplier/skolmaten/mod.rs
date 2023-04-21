use std::{iter, ops::RangeInclusive};

use futures::{
    stream::{self, StreamExt},
    TryFutureExt, TryStreamExt,
};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use stor::{
    meal::sanitize_meal_value,
    menu::{Patch, Supplier},
};
use time::{Date, Month};
use tracing::{error, instrument};

use crate::{Error, Result};

use super::ListDays;

/// Maximum number of concurrent HTTP requests when crawling. For comparison,
/// Firefox allows 7 concurrent requests. There is virtually no improvement for
/// values above 64, and 32 is just marginally slower (and half the memory
/// usage).
const CONCURRENT_REQUESTS: usize = 32;

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Debug, Clone, Copy, Deserialize)]
struct Location {
    longitude: f64,
    latitude: f64,
}

impl From<Location> for geo::Point {
    fn from(
        Location {
            longitude,
            latitude,
        }: Location,
    ) -> Self {
        geo::Point::new(longitude, latitude)
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Station {
    id: u64,
    name: String,
    // image_url: Option<String>,
    district: Option<District>,
    location: Option<Location>,
}

#[derive(Deserialize, Debug)]
struct StationsResponse {
    stations: Vec<Station>,
}

impl Station {
    fn district_name(&self) -> Option<&str> {
        self.district.as_ref().map(|d| d.name.as_str())
    }

    /// If `district_name` is `None`, the Station's internal district name will be
    /// used, provided it exists.
    fn to_menu(&self, district_name: Option<&str>) -> Option<stor::Menu> {
        let district_name = district_name.or_else(|| self.district_name())?;

        let mut menu = stor::Menu::from_supplier(
            Supplier::Skolmaten,
            self.id.to_string(),
            format!("{}, {}", self.name.trim(), district_name),
        );

        menu.location = self.location.map(Into::into);

        Some(menu)
    }
}

#[instrument(level = "debug", skip(client))]
async fn list_provinces(client: &Client) -> Result<Vec<Province>> {
    let res = fetch(client, "provinces")
        .await?
        .json::<ProvincesResponse>()
        .await?;

    Ok(res.provinces)
}

#[instrument(level = "debug", skip(client))]
async fn list_districts_in_province(client: &Client, province_id: u64) -> Result<Vec<District>> {
    let res = fetch(client, &format!("districts?province={province_id}"))
        .await?
        .json::<DistrictsResponse>()
        .await?;

    Ok(res.districts)
}

#[instrument(level = "debug", skip(client))]
async fn list_stations_in_district(client: &Client, district_id: u64) -> Result<Vec<Station>> {
    let res = fetch(client, &format!("stations?district={district_id}"))
        .await?
        .json::<StationsResponse>()
        .await?;

    Ok(res.stations)
}

#[instrument(name = "skolmaten::list_menus", err, skip(client))]
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
                .filter_map(|s| s.to_menu(Some(&district_name))),
        );
    }

    Ok(menus)
}

#[derive(Deserialize, Debug, Clone)]
struct Meal {
    value: String,
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

        let meals = self
            .meals?
            .into_iter()
            .filter_map(|m| sanitize_meal_value(&m.value))
            .collect();

        Some(stor::Day::new(date, meals))
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Week {
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
    station: Station,
    // id: u64,
    // bulletins: Vec<Bulletin>,
}

#[derive(PartialEq, Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct WeekSpan {
    year: i32,
    /// Starting year of the span.
    week_of_year: u8,
    /// Total number of weeks.
    count: u8,
}

#[instrument(level = "debug", skip(client))]
async fn fetch_menu(client: &Client, station: u64, span: WeekSpan) -> Result<Menu> {
    #[derive(Debug, Serialize)]
    struct Query {
        #[serde(flatten)]
        span: WeekSpan,
        station: u64,
    }

    #[derive(Deserialize, Debug)]
    struct Response {
        menu: Menu,
    }

    let path = format!(
        "menu?{}",
        serde_urlencoded::to_string(Query { span, station }).unwrap()
    );

    let res = fetch(client, &path).await?;
    let status = res.status();
    let res = res.json::<Response>().await.map_err(|e| {
        if status != StatusCode::NOT_FOUND {
            error!("{}", e);
        }

        Error::MenuNotFound
    })?;

    Ok(res.menu)
}

/// Generate a series of queries because the Skolmaten API cannot handle
/// more than one year per request.
fn week_spans(range: RangeInclusive<Date>) -> impl Iterator<Item = WeekSpan> {
    let mut span_start = *range.start();

    iter::from_fn(move || {
        if span_start > *range.end() {
            return None;
        }

        let year = span_start.year();

        let span_end = if range.end().year() == year {
            *range.end()
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
#[instrument(skip(client), fields(?dates))]
pub async fn list_days(
    client: &Client,
    station: u64,
    dates: RangeInclusive<Date>,
) -> Result<ListDays> {
    let mut results = stream::iter(week_spans(dates.clone()))
        .map(|span| fetch_menu(client, station, span))
        .buffer_unordered(4);

    let mut menu = Patch::default();
    let mut days = vec![];

    while let Some(res) = results.try_next().await? {
        days.extend(
            res.weeks
                .into_iter()
                .flat_map(|w| w.days.into_iter().filter_map(Day::normalize))
                .filter(|d| dates.contains(&d.date)),
        );

        menu.location = res.station.location.map(Into::into).or(menu.location);
    }

    days.sort();
    days.dedup_by_key(|d| d.date);

    Ok(ListDays { menu, days })
}

#[instrument(level = "debug", err)]
async fn fetch(client: &Client, path: &str) -> reqwest::Result<reqwest::Response> {
    let url = format!("https://skolmaten.se/api/4/{path}");

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

    use crate::supplier::ListDays;

    use super::WeekSpan;

    #[tokio::test]
    async fn list_menus() {
        let menus = super::list_menus(&Client::new()).await.unwrap();

        assert!(menus.len() > 5000);
    }

    #[tokio::test]
    async fn list_days() {
        let first_day = OffsetDateTime::now_utc().to_timezone(crate::TZ).date();
        let last_day = first_day + Duration::weeks(2);

        let ListDays { menu, days } =
            super::list_days(&Client::new(), 4889403990212608, first_day..=last_day)
                .await
                .unwrap();

        assert!(!menu.is_empty());
        assert!(!days.is_empty());
    }

    #[test]
    fn week_spans() {
        let mut it = super::week_spans(date!(2020 - 08 - 01)..=date!(2021 - 03 - 01));
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
