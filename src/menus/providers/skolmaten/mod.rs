mod days;
mod fetch;

use std::time::Instant;

use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde::Deserialize;

use crate::menus::{LocalDay, LocalMenu, Provider};

use self::{days::list_days, fetch::fetch_json};

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

#[derive(Deserialize, Debug)]
struct District {
    id: u64,
    name: String,
}

#[derive(Deserialize, Debug)]
struct DistrictsResponse {
    districts: Vec<District>,
}

#[derive(Deserialize, Debug)]
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

async fn list_provinces(client: &Client) -> reqwest::Result<Vec<Province>> {
    let res = fetch_json::<ProvincesResponse>(&client, "provinces")
        .await
        .unwrap();

    Ok(res.provinces)
}

async fn list_districts_in_province(
    client: &Client,
    province_id: u64,
) -> reqwest::Result<Vec<District>> {
    let res =
        fetch_json::<DistrictsResponse>(client, &format!("districts?province={}", province_id))
            .await?;

    Ok(res.districts)
}

async fn list_stations_in_district(
    client: &Client,
    district_id: u64,
) -> reqwest::Result<Vec<Station>> {
    let res = fetch_json::<StationsResponse>(client, &format!("stations?district={}", district_id))
        .await?;

    Ok(res.stations)
}

#[async_trait]
impl Provider for Skolmaten {
    fn id() -> String {
        "skolmaten".to_owned()
    }

    fn name() -> String {
        "Skolmaten".to_owned()
    }

    async fn list_menus() -> Vec<LocalMenu> {
        let before_crawl = Instant::now();

        let client = Client::new();

        let provinces = list_provinces(&client).await.unwrap();

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

        menus
    }

    async fn query_menu(menu_id: String) -> Option<LocalMenu> {
        todo!()
    }

    async fn list_days(menu_id: String) -> Vec<LocalDay> {
        let client = Client::new();

        let station_id = menu_id.parse::<u64>().unwrap();

        let days = list_days(&client, station_id).await.unwrap();

        days
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn list_menus() {
        let menus = Skolmaten::list_menus().await;
        
        assert!(menus.len() > 5000);

        for menu in menus {
            assert!(!menu.title.to_lowercase().contains("info"));
        }
    }
}
