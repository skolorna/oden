use butler_lib::menus::id::MenuId;
use chrono::{DateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable};

use crate::schema::menus;

#[derive(Identifiable, Queryable, PartialEq, Eq, Debug)]
pub struct Menu {
    pub id: MenuId,
    pub title: String,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[table_name = "menus"]
pub struct NewMenu {
    pub id: MenuId,
    pub title: String,
}

impl From<butler_lib::types::menu::Menu> for NewMenu {
    fn from(menu: butler_lib::types::menu::Menu) -> Self {
        Self {
            id: menu.id,
            title: menu.title,
        }
    }
}
