pub mod providers;

use async_trait::async_trait;
use chrono::NaiveDate;
use serde::Serialize;

#[derive(Serialize)]
pub struct LocalMenu {
    id: String,
    title: String,
}

#[derive(Serialize)]
pub struct LocalMeal {
    value: String,
}

#[derive(Serialize)]
pub struct LocalDay {
    meals: Vec<LocalMeal>,
    /// Time zones aren't really relevant here.
    date: NaiveDate,
}

#[async_trait]
pub trait Provider {
    fn id() -> String;

    fn name() -> String;

    async fn list_menus() -> Vec<LocalMenu>;

    async fn query_menu(menu_id: String) -> Option<LocalMenu>;

    async fn list_days(menu_id: String) -> Vec<LocalDay>;
}
