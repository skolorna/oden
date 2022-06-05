#[macro_use]
pub mod mashie;
pub mod supplier;

use chrono::NaiveDate;
use futures::{stream, StreamExt};
use strum::IntoEnumIterator;
use tracing::{debug, instrument};

use crate::{errors::Result, Day, Meal, Menu, MenuSlug};

use self::supplier::Supplier;

/// List all the menus everywhere (from all suppliers).
#[instrument]
pub async fn list_menus(concurrent: usize) -> Result<Vec<Menu>> {
    debug!("listing menus");

    let mut menus = stream::iter(Supplier::iter())
        .map(|p| async move { p.list_menus().await })
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

pub async fn list_days(
    menu_slug: &MenuSlug,
    first: NaiveDate,
    last: NaiveDate,
) -> Result<Vec<Day>> {
    menu_slug
        .supplier
        .list_days(&menu_slug.local_id, first, last)
        .await
}
