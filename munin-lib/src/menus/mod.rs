#[macro_use]
pub mod mashie;
pub mod meal;
pub mod supplier;

use chrono::NaiveDate;
use futures::{stream, StreamExt};
use strum::IntoEnumIterator;

use crate::{
    errors::MuninResult,
    types::{day::Day, menu::Menu, slug::MenuSlug},
};

use self::{meal::Meal, supplier::Supplier};

/// List all the menus everywhere (from all suppliers).
pub async fn list_menus(concurrent: usize) -> MuninResult<Vec<Menu>> {
    let mut menus = stream::iter(Supplier::iter())
        .map(|p| async move { p.list_menus().await })
        .buffer_unordered(concurrent)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<MuninResult<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<Menu>>();

    menus.sort_by(|a, b| a.title().cmp(b.title()));

    Ok(menus)
}

pub async fn query_menu(menu_slug: &MenuSlug) -> MuninResult<Menu> {
    menu_slug.supplier.query_menu(&menu_slug.local_id).await
}

pub async fn list_days(
    menu_slug: &MenuSlug,
    first: NaiveDate,
    last: NaiveDate,
) -> MuninResult<Vec<Day>> {
    menu_slug
        .supplier
        .list_days(&menu_slug.local_id, first, last)
        .await
}
