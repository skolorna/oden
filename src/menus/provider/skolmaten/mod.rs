pub mod days;
pub mod fetch;

use std::time::Instant;

use chrono::NaiveDate;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde::Deserialize;

use crate::{
    errors::{BadInputError, Error, NotFoundError, Result},
    menus::{id::MenuID, provider::Provider, Day, Menu},
};

use self::{days::query_station, fetch::fetch};

/// Maximum number of concurrent HTTP requests when crawling. For comparison,
/// Firefox allows 7 concurrent requests. There is virtually no improvement for
/// values above 64, and 32 is just marginally slower.
const CONCURRENT_REQUESTS: usize = 32;

#[derive(Deserialize, Debug)]
struct Province {
    id: u64,
    name: String,
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
                MenuID::new(Provider::Skolmaten, self.id.to_string()),
                format!("{}, {}", self.name.trim(), district_name),
            ))
        }
    }
}

fn parse_menu_id(menu_id: &str) -> Result<u64> {
    menu_id
        .parse::<u64>()
        .map_err(|e| Error::BadInputError(BadInputError::ParseIntError(e)))
}

async fn list_provinces(client: &Client) -> Result<Vec<Province>> {
    let res = fetch(&client, "provinces")
        .await?
        .json::<ProvincesResponse>()
        .await?;

    Ok(res.provinces)
}

async fn list_districts_in_province(client: &Client, province_id: u64) -> Result<Vec<District>> {
    let res = fetch(client, &format!("districts?province={}", province_id))
        .await?
        .json::<DistrictsResponse>()
        .await?;

    Ok(res.districts)
}

async fn list_stations_in_district(client: &Client, district_id: u64) -> Result<Vec<Station>> {
    let res = fetch(client, &format!("stations?district={}", district_id))
        .await?
        .json::<StationsResponse>()
        .await?;

    Ok(res.stations)
}

pub(super) async fn list_menus() -> Result<Vec<Menu>> {
    let before_crawl = Instant::now();

    let client = Client::new();

    let provinces = list_provinces(&client).await?;

    let districts: Vec<District> = stream::iter(provinces)
        .map(|province| {
            let client = &client;
            async move {
                let districts = list_districts_in_province(&client, province.id)
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
                list_stations_in_district(&client, district.id)
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

    println!("{}ms crawl", before_crawl.elapsed().as_millis());

    Ok(menus)
}

pub(super) async fn query_menu(menu_id: &str) -> Result<Menu> {
    let client = Client::new();
    let station_id = parse_menu_id(menu_id)?;

    let station = query_station(&client, station_id).await?;
    let menu = station.to_menu().ok_or(NotFoundError::MenuNotFoundError)?;

    Ok(menu)
}

pub(super) async fn list_days(
    menu_id: &str,
    first: NaiveDate,
    last: NaiveDate,
) -> Result<Vec<Day>> {
    let client = Client::new();
    let station_id = parse_menu_id(menu_id)?;
    days::list_days(&client, station_id, first, last).await
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    #[actix_rt::test]
    async fn list_menus_test() {
        let menus = list_menus().await.unwrap();

        assert!(menus.len() > 5000);

        for menu in menus {
            assert!(!menu.title.to_lowercase().contains("info"));
        }
    }

    #[test]
    fn parse_station_id_test() {
        assert_eq!(parse_menu_id("1234").unwrap(), 1234);
        assert!(parse_menu_id("aaf705").is_err());
    }

    #[actix_rt::test]
    async fn query_menu_test() {
        let menu = query_menu("4791333780717568").await.unwrap();
        assert_eq!(menu.title, "Stråtjära förskola, Söderhamns kommun");
        assert!(query_menu("0").await.is_err());
        assert!(query_menu("5236876508135424").await.is_err()); // Invalid station name
    }

    #[actix_rt::test]
    async fn list_days_test() {
        let first_day = chrono::offset::Local::now().date().naive_local();
        let last_day = first_day + Duration::weeks(2);

        let days = list_days("4791333780717568", first_day, last_day)
            .await
            .unwrap();

        assert!(days.len() > 0);
    }
}
