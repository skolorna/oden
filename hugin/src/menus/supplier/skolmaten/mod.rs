mod days;
mod fetch;

use chrono::NaiveDate;
use futures::{
    stream::{self, StreamExt},
    TryFutureExt,
};
use reqwest::Client;
use serde::Deserialize;
use tracing::instrument;

use crate::{errors::Result, menus::supplier::Supplier, Day, Menu, MenuSlug};

use fetch::fetch;

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

#[instrument(err)]
pub(super) async fn list_menus() -> Result<Vec<Menu>> {
    let client = Client::new();

    let provinces = list_provinces(&client).await?;

    let mut districts = Vec::new();

    let mut districts_stream = stream::iter(provinces)
        .map(|province| list_districts_in_province(&client, province.id))
        .buffer_unordered(CONCURRENT_REQUESTS);

    while let Some(res) = districts_stream.next().await {
        districts.extend(res?);
    }

    let mut menus = Vec::new();

    let mut menus_stream = stream::iter(districts)
        .map(|district| {
            list_stations_in_district(&client, district.id)
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

#[instrument(fields(%first, %last))]
pub(super) async fn list_days(
    menu_slug: u64,
    first: NaiveDate,
    last: NaiveDate,
) -> Result<Vec<Day>> {
    let client = Client::new();
    days::list_days(&client, menu_slug, first, last).await
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    #[tokio::test]
    async fn list_menus_test() {
        let menus = list_menus().await.unwrap();

        assert!(menus.len() > 5000);

        for menu in menus {
            assert!(!menu.title().to_lowercase().contains("info"));
        }
    }

    #[tokio::test]
    async fn list_days_test() {
        let first_day = chrono::offset::Local::now().date().naive_local();
        let last_day = first_day + Duration::weeks(2);

        let days = list_days(4889403990212608, first_day, last_day)
            .await
            .unwrap();

        assert!(!days.is_empty());
    }
}
