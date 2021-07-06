mod days;
mod fetch;

use std::time::Instant;

use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde::Deserialize;

use crate::{
    errors::{BadInputError, Error, NotFoundError, Result},
    menus::{providers::skolmaten::days::query_station, LocalDay, LocalMenu, Provider, ProviderID},
};

use self::{days::list_days, fetch::fetch};

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
    fn to_local_menu(&self, district_name: &str) -> Option<LocalMenu> {
        if self.name.to_lowercase().contains("info") {
            None
        } else {
            Some(LocalMenu {
                title: format!("{}, {}", self.name, district_name),
                id: self.id.to_string(),
            })
        }
    }
}

pub struct Skolmaten {}

impl Skolmaten {
    fn parse_menu_id(menu_id: &str) -> Result<u64> {
        menu_id
            .parse::<u64>()
            .map_err(|e| Error::BadInputError(BadInputError::ParseIntError(e)))
    }
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

#[async_trait]
impl Provider for Skolmaten {
    fn id() -> ProviderID {
        "skolmaten".to_owned()
    }

    fn name() -> String {
        "Skolmaten".to_owned()
    }

    async fn list_menus() -> Result<Vec<LocalMenu>> {
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

        let menus: Vec<LocalMenu> = stream::iter(districts)
            .map(|district| {
                let client = &client;
                async move {
                    list_stations_in_district(&client, district.id)
                        .await
                        .unwrap()
                        .into_iter()
                        .map(|station| station.to_local_menu(&district.name))
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

        println!("{}ms", before_crawl.elapsed().as_millis());

        Ok(menus)
    }

    async fn query_menu(menu_id: &str) -> Result<LocalMenu> {
        let client = Client::new();
        let station_id = Skolmaten::parse_menu_id(menu_id)?;

        let station = query_station(&client, station_id).await?;
        let menu = station
            .to_local_menu()
            .ok_or(NotFoundError::MenuNotFoundError)?;

        Ok(menu)
    }

    async fn list_days(menu_id: &str) -> Result<Vec<LocalDay>> {
        let client = Client::new();
        let station_id = Skolmaten::parse_menu_id(menu_id)?;

        let days = list_days(&client, station_id).await?;

        Ok(days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn list_menus() {
        let menus = Skolmaten::list_menus().await.unwrap();

        assert!(menus.len() > 5000);

        for menu in menus {
            assert!(!menu.title.to_lowercase().contains("info"));
        }
    }

    #[actix_rt::test]
    async fn list_days() {
        let days = Skolmaten::list_days("4791333780717568").await.unwrap();

        assert!(days.len() > 0);
    }
}
