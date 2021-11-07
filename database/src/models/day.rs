use butler_lib::types;
use butler_lib::{menus::id::MenuId, types::day::DayId};
use chrono::{NaiveDate};

use crate::models::menu::Menu;
use crate::schema::days;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Menu)]
#[table_name = "days"]
pub struct Day {
    pub id: DayId,
    pub date: NaiveDate,
    pub menu_id: MenuId,
}

#[derive(Debug, Insertable)]
#[table_name = "days"]
pub struct NewDay {
    id: DayId,
    date: NaiveDate,
    meals: String,
    menu_id: MenuId,
}

impl NewDay {
    pub fn from_day(day: types::day::Day, menu: MenuId) -> Self {
        Self {
            id: day.get_id(&menu),
            date: day.date,
            menu_id: menu,
            meals: day
                .meals
                .into_iter()
                .map(|m| m.value)
                .collect::<Vec<String>>()
                .join("\n"),
        }
    }
}
