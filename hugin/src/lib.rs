#![doc = include_str!("../README.md")]
#![warn(missing_debug_implementations, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

mod day;
mod errors;
mod mashie;
mod meal;
mod menu;
mod supplier;
mod util;

use chrono::NaiveDate;
use futures::{stream, StreamExt};

use reqwest::Client;
use strum::IntoEnumIterator;
use tracing::{debug, instrument};

pub use day::*;
pub use errors::*;
pub use meal::*;
pub use menu::*;
pub use supplier::*;

/// List all the menus everywhere (from all suppliers).
#[instrument]
pub async fn list_menus(concurrent: usize) -> Result<Vec<Menu>> {
    debug!("listing menus");

    let client = Client::new();

    let mut menus = stream::iter(Supplier::iter())
        .map(|s| {
            let client = client.clone();
            async move { s.list_menus(&client).await }
        })
        .buffer_unordered(concurrent)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<Menu>>();

    menus.sort_by(|a, b| a.title().cmp(b.title()));

    Ok(menus)
}

#[instrument(skip(client), fields(%menu_slug, %first, %last))]
pub async fn list_days(
    client: &Client,
    menu_slug: &MenuSlug,
    first: NaiveDate,
    last: NaiveDate,
) -> Result<Vec<Day>> {
    menu_slug
        .supplier
        .list_days(client, &menu_slug.local_id, first, last)
        .await
}
