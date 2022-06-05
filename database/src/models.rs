use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{days, menus};
use crate::smaztext::SmazText;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Menu)]
#[table_name = "days"]
#[primary_key(date, menu_id)]
pub struct Day {
    pub date: NaiveDate,
    pub meals: SmazText,
    pub menu_id: MenuId,
}

#[derive(Debug, Insertable)]
#[table_name = "days"]
pub struct NewDay {
    date: NaiveDate,
    menu_id: MenuId,
    meals: SmazText,
}

impl NewDay {
    pub fn from_day(day: munin_lib::Day, menu_id: MenuId) -> Self {
        let meals = day
            .meals
            .into_iter()
            .map(|m| m.value)
            .collect::<Vec<_>>()
            .join("\n");

        Self {
            date: day.date,
            menu_id,
            meals: meals.into(),
        }
    }
}

impl From<Day> for munin_lib::Day {
    fn from(d: Day) -> Self {
        Self {
            date: d.date,
            meals: d.meals.into(),
        }
    }
}

pub type MenuId = Uuid;

#[derive(Identifiable, Queryable, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Menu {
    pub id: MenuId,
    pub title: String,
    pub slug: munin_lib::MenuSlug,
    pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(feature = "meilisearch-sdk")]
impl crate::MeiliIndexable for Menu {
    const MEILI_INDEX: &'static str = "menus";
}

#[derive(Debug, Insertable)]
#[table_name = "menus"]
pub struct NewMenu {
    pub id: Uuid,
    pub slug: munin_lib::MenuSlug,
    pub title: String,
}

impl From<munin_lib::Menu> for NewMenu {
    fn from(menu: munin_lib::Menu) -> Self {
        Self {
            id: menu.get_uuid(),
            slug: menu.slug,
            title: menu.title,
        }
    }
}
