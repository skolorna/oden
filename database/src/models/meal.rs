use butler_lib::types;
use chrono::NaiveDate;
use sha2::{Digest, Sha256};

use crate::models::menu::Menu;
use crate::schema::meals;

use super::menu::MenuId;

pub type MealId = Vec<u8>;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Menu)]
#[table_name = "meals"]
pub struct Meal {
    pub id: MealId,
    pub date: NaiveDate,
    pub value: String,
    pub menu_id: MenuId,
}

#[derive(Debug, Insertable)]
#[table_name = "meals"]
pub struct NewMeal {
    id: MealId,
    date: NaiveDate,
    menu_id: MenuId,
    value: String,
}

impl NewMeal {
    pub fn from_day(day: types::day::Day, menu_id: MenuId) -> Vec<Self> {
        let date = day.date;

        day.meals
            .into_iter()
            .map(|meal| {
                let mut hasher = Sha256::new();

                hasher.update(menu_id.to_be_bytes());
                hasher.update(date.to_string().as_bytes());
                hasher.update(&meal.value);

                // TODO: This might cause collisions
                let id = hasher.finalize()[..8].to_vec();

                NewMeal {
                    id,
                    date,
                    value: meal.value,
                    menu_id,
                }
            })
            .collect()
    }
}
