use butler_lib::types;
use chrono::NaiveDate;

use crate::models::menu::Menu;
use crate::schema::meals;

use super::menu::MenuId;

pub type MealId = i32;

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
    date: NaiveDate,
    menu_id: MenuId,
    value: String,
}

impl NewMeal {
    pub fn from_day(day: types::day::Day, menu_id: MenuId) -> Vec<Self> {
        let date = day.date;

        day.meals
            .into_iter()
            .map(|meal| NewMeal {
                date,
                value: meal.value,
                menu_id,
            })
            .collect()
    }
}
