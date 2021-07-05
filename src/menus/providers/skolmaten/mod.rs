use std::time::Instant;

use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize};

use crate::menus::{LocalDay, LocalMenu, Provider};

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
pub struct Station {
    id: u64,
    name: String,
}

impl Station {
    fn to_local_menu(&self, district_name: &str) -> LocalMenu {
        LocalMenu {
            title: format!("{}, {}", self.name, district_name),
            id: self.id.to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct StationsResponse {
    stations: Vec<Station>,
}

pub struct Skolmaten {}

async fn fetch_json<T: DeserializeOwned>(client: &Client, path: &str) -> reqwest::Result<T> {
    let url = format!("https://skolmaten.se/api/4/{}", path);

    // let body = reqwest::get(url).await?.json().await?;

    let res = client
        .get(&url)
        .header("API-Version", "4.0")
        .header("Client-Token", "web")
        .header("Client-Version-Token", "web")
        .header("Locale", "sv_SE")
        .send()
        .await?;

    let body = res.json::<T>().await?;

    Ok(body)
}

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

        println!("fetched {} districts", districts.len());

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
            .collect();

        println!("{}ms", before_crawl.elapsed().as_millis());

        menus
    }

    async fn query_menu(menu_id: String) -> Option<LocalMenu> {
        todo!()
    }

    async fn list_days(menu_id: String) -> Vec<LocalDay> {
        todo!()
    }
}
