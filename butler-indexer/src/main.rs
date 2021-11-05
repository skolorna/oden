use std::sync::Arc;

use atomic_counter::{AtomicCounter, RelaxedCounter};
use butler_lib::menus::{day::Day, list_days, list_menus, Menu};
use chrono::{Duration, TimeZone, Utc};
use chrono_tz::Europe::Stockholm;
use futures::{stream, StreamExt};
use meilisearch_sdk::{client::Client, document::Document};
use serde::{Deserialize, Serialize};
use tracing::info;

// #[tokio::main]
// async fn main() {
//     let client = Client::new("http://localhost:7700", "");

// let menus_index = client.get_or_create("menus").await.unwrap();
//     menus_index.set_searchable_attributes(&["title"]).await.unwrap();

//     // reading and parsing the file
//     let menus_docs = list_menus(4).await.unwrap();

//     menus_index.delete_all_documents().await.unwrap();
//     menus_index.add_documents(&menus_docs, None).await.unwrap();
// }

#[derive(Debug, Serialize, Deserialize)]
struct MealDoc {
    id: String,
    value: String,
}

impl Document for MealDoc {
    type UIDType = String;

    fn get_uid(&self) -> &Self::UIDType {
        &self.id
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let client = Client::new("http://localhost:7700", "");
    let menus_index = client.get_or_create("menus").await.unwrap();
    menus_index
        .set_searchable_attributes(&["title"])
        .await
        .unwrap();

    let menus = list_menus(4).await.unwrap();
    menus_index.add_documents(&menus, None).await.unwrap();
    let hmmm = menus.len();
    let menus_count = Arc::new(RelaxedCounter::new(0));

    info!("found {} menus", menus.len());

    let utc = Utc::now().naive_utc();
    let first = Stockholm.from_utc_datetime(&utc).date().naive_local();
    let last = first + Duration::days(90);

    let days: Vec<Vec<Day>> = stream::iter(menus)
        .map(|m| {
            let c = menus_count.clone();
            async move {
                let days = list_days(m.id(), first, last).await.unwrap_or_default();
                let c = c.inc();
                info!(
                    "{}/{}: fetched {} days -- {}",
                    c,
                    hmmm,
                    days.len(),
                    m.title()
                );
                days
            }
        })
        .buffer_unordered(100)
        .collect::<Vec<_>>()
        .await;

    let meals: Vec<MealDoc> = days
        .into_iter()
        .flatten()
        .flat_map(|d| {
            d.meals()
                .iter()
                .map(|m| MealDoc {
                    value: m.value().to_owned(),
                    id: base64::encode_config(m.value(), base64::URL_SAFE_NO_PAD),
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let meals_index = client.get_or_create("meals").await.unwrap();

    meals_index.add_documents(&meals, None).await.unwrap();

    // info!("{} days in total", days.into_iter().flatten().collect::<Vec<Day>>().len());

    // for menu in menus {

    //     let days = ;

    //     days_total += days.len();

    //     println!("{} days in total", days_total);
    // }
}
