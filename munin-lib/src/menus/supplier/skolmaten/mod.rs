mod days;
mod fetch;

use chrono::NaiveDate;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde::Deserialize;

use crate::{
    errors::{MuninError, MuninResult},
    menus::{supplier::Supplier, Day, Menu, MenuSlug},
};

use self::{days::query_station, fetch::fetch};

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

async fn list_provinces(client: &Client) -> MuninResult<Vec<Province>> {
    let res = fetch(client, "provinces")
        .await?
        .json::<ProvincesResponse>()
        .await?;

    Ok(res.provinces)
}

async fn list_districts_in_province(
    client: &Client,
    province_id: u64,
) -> MuninResult<Vec<District>> {
    let res = fetch(client, &format!("districts?province={}", province_id))
        .await?
        .json::<DistrictsResponse>()
        .await?;

    Ok(res.districts)
}

async fn list_stations_in_district(client: &Client, district_id: u64) -> MuninResult<Vec<Station>> {
    let res = fetch(client, &format!("stations?district={}", district_id))
        .await?
        .json::<StationsResponse>()
        .await?;

    Ok(res.stations)
}

pub(super) async fn list_menus() -> MuninResult<Vec<Menu>> {
    let client = Client::new();

    let provinces = list_provinces(&client).await?;

    let districts: Vec<District> = stream::iter(provinces)
        .map(|province| {
            let client = &client;
            async move {
                let districts = list_districts_in_province(client, province.id)
                    .await
                    .unwrap();

                districts
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect();

    let menus: Vec<Menu> = stream::iter(districts)
        .map(|district| {
            let client = &client;
            async move {
                list_stations_in_district(client, district.id)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|station| station.to_menu(&district.name))
                    .collect::<Vec<_>>()
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .flatten()
        .collect();

    Ok(menus)
}

pub(super) async fn query_menu(menu_slug: u64) -> MuninResult<Menu> {
    let client = Client::new();

    let station = query_station(&client, menu_slug).await?;
    let menu = station.to_menu().ok_or(MuninError::MenuNotFound)?;

    Ok(menu)
}

pub(super) async fn list_days(
    menu_slug: u64,
    first: NaiveDate,
    last: NaiveDate,
) -> MuninResult<Vec<Day>> {
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
    async fn query_menu_test() {
        let menu = query_menu(4791333780717568).await.unwrap();
        assert_eq!(menu.title(), "Stråtjära förskola, Söderhamns kommun");
        assert!(query_menu(0).await.is_err());
        assert!(query_menu(5236876508135424).await.is_err()); // Invalid station name
    }

    #[tokio::test]
    async fn list_days_test() {
        let first_day = chrono::offset::Local::now().date().naive_local();
        let last_day = first_day + Duration::weeks(2);

        let days = list_days(4791333780717568, first_day, last_day)
            .await
            .unwrap();

        assert!(!days.is_empty());
    }
}
