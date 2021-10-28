pub mod day;
pub mod id;

#[macro_use]
pub mod mashie;
pub mod meal;
pub mod supplier;

use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::errors::ButlerResult;

use self::{
    day::Day,
    id::MenuID,
    meal::Meal,
    supplier::{Supplier, SupplierInfo},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Menu {
    id: MenuID,
    title: String,
    supplier: SupplierInfo,
}

impl Menu {
    pub fn new(id: MenuID, title: String) -> Self {
        Self {
            supplier: id.supplier.info(),
            id,
            title,
        }
    }

    pub fn id(&self) -> &MenuID {
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
