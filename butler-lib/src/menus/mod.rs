pub mod day;
pub mod id;

#[macro_use]
pub mod mashie;
pub mod meal;
pub mod supplier;

use chrono::NaiveDate;
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::errors::ButlerResult;

use self::{
    day::Day,
    id::MenuId,
    meal::Meal,
    supplier::{Supplier, SupplierInfo},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Menu {
    id: MenuId,
    title: String,
    supplier: SupplierInfo,
}

impl Menu {
    pub fn new(id: MenuId, title: String) -> Self {
        Self {
            supplier: id.supplier.info(),
            id,
            title,
        }
    }

    pub fn id(&self) -> &MenuId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn supplier(&self) -> &SupplierInfo {
        &self.supplier
    }
}

/// List all the menus everywhere (from all suppliers).
pub async fn list_menus(concurrent: usize) -> ButlerResult<Vec<Menu>> {
    let mut menus = stream::iter(Supplier::iter())
        .map(|p| async move { p.list_menus().await })
        .buffer_unordered(concurrent)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<ButlerResult<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<Menu>>();

    menus.sort_by(|a, b| a.title.cmp(&b.title));

    Ok(menus)
}

pub async fn query_menu(menu_id: &MenuId) -> ButlerResult<Menu> {
    menu_id.supplier.query_menu(&menu_id.local_id).await
}

pub async fn list_days(
    menu_id: &MenuId,
    first: NaiveDate,
    last: NaiveDate,
) -> ButlerResult<Vec<Day>> {
    menu_id
        .supplier
        .list_days(&menu_id.local_id, first, last)
        .await
}
