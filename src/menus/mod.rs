pub mod id;
pub mod providers;

use chrono::NaiveDate;
use providers::skolmaten::days::SkolmatenMeal;
use serde::Serialize;

use self::id::MenuID;

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
