pub mod id;
pub mod provider;
pub mod skolmaten;

use chrono::NaiveDate;
use serde::Serialize;
use skolmaten::days::SkolmatenMeal;

use self::{id::MenuID, provider::Provider};
use crate::errors::Result;

#[derive(Serialize, Debug)]
pub struct Meal {
    value: String,
}

impl From<SkolmatenMeal> for Meal {
    fn from(meal: SkolmatenMeal) -> Self {
        Self { value: meal.value }
    }
}

#[derive(Serialize, Debug)]
pub struct Day {
    /// Time zones aren't really relevant here.
    date: NaiveDate,
    meals: Vec<Meal>,
}

#[derive(Serialize)]
pub struct Menu {
    id: MenuID,
    title: String,
}

impl Menu {
    pub fn new(id: MenuID, title: &str) -> Self {
        Self {
            id,
            title: title.to_owned(),
        }
    }
}

pub async fn list_menus() -> Result<Vec<Menu>> {
    let menus = skolmaten::list_menus().await?;

    Ok(menus)
}

pub async fn query_menu(menu_id: &MenuID) -> Result<Menu> {
    match menu_id.provider {
        Provider::Skolmaten => skolmaten::query_menu(&menu_id.local_id).await,
    }
}

pub async fn list_days(menu_id: &MenuID) -> Result<Vec<Day>> {
    match menu_id.provider {
        Provider::Skolmaten => skolmaten::list_days(&menu_id.local_id).await,
    }
}
