pub mod providers;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LocalMenu {
    id: String,
    title: String,
}

#[derive(Serialize, Deserialize)]
pub struct Meal {
    value: String,
}

#[derive(Serialize, Deserialize)]
pub struct LocalDay {
    meals: Vec<Meal>,
    date: String,
}

#[async_trait]
pub trait Provider {
    fn id() -> String;

    fn name() -> String;

    async fn list_menus() -> Vec<LocalMenu>;

    async fn query_menu(menu_id: String) -> Option<LocalMenu>;

    async fn list_days(menu_id: String) -> Vec<LocalDay>;
}
