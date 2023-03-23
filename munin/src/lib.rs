use std::ops::RangeInclusive;

use futures::{stream, StreamExt};
use reqwest::Client;
use stor::{menu::Supplier, Menu};
use strum::IntoEnumIterator;
use supplier::ListDays;
use thiserror::Error;
use time::Date;
use tracing::{debug, instrument};

use crate::supplier::{kleins, matilda, mpi, sabis, skolmaten, sodexo};

pub mod geosearch;
pub mod index;
mod mashie;
pub mod supplier;
mod util;

pub type Result<T, E = anyhow::Error> = core::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("menu not found")]
    MenuNotFound,
    #[error("unexpected scrape error")]
    ScrapeError,
}

impl Error {
    fn scrape_error_with_context(_message: &str, _context: Option<&str>) -> Self {
        Self::ScrapeError
    }

    fn scrape_error(message: &str) -> Self {
        Self::scrape_error_with_context(message, None)
    }
}

pub const TZ: &time_tz::Tz = time_tz::timezones::db::europe::STOCKHOLM;

pub const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_REPOSITORY"),
    ")"
);

#[instrument]
pub async fn list_menus(concurrent: usize) -> Result<Vec<Menu>> {
    debug!("listing menus");

    let client = Client::new();

    let menus = stream::iter(Supplier::iter())
        .map(|s| {
            let client = client.clone();
            async move {
                match s {
                    Supplier::Skolmaten => skolmaten::list_menus(&client).await,
                    Supplier::Sodexo => sodexo::list_menus(&client).await,
                    Supplier::Mpi => mpi::list_menus(&client).await,
                    Supplier::Kleins => kleins::list_menus(&client).await,
                    Supplier::Sabis => Ok(Vec::new()),
                    Supplier::Matilda => matilda::list_menus(&client).await,
                }
            }
        })
        .buffer_unordered(concurrent)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<Menu>>();

    Ok(menus)
}

#[instrument(skip(client), fields(?supplier, %supplier_reference, ?range))]
pub async fn list_days(
    client: &Client,
    supplier: Supplier,
    supplier_reference: &str,
    range: RangeInclusive<Date>,
) -> Result<ListDays> {
    let first = *range.start();
    let last = *range.end();

    match supplier {
        Supplier::Skolmaten => {
            skolmaten::list_days(client, supplier_reference.parse().unwrap(), range).await
        }
        Supplier::Sodexo => sodexo::list_days(client, supplier_reference, first, last).await,
        Supplier::Mpi => mpi::list_days(client, supplier_reference, first, last).await,
        Supplier::Kleins => kleins::list_days(client, supplier_reference, first, last).await,
        Supplier::Sabis => sabis::list_days(client, supplier_reference).await,
        Supplier::Matilda => {
            matilda::list_days(client, &supplier_reference.parse().unwrap(), first, last).await
        }
    }
}
