use munin_lib::menus::id::MenuSlug;
use chrono::{DateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::schema::menus;

pub type MenuId = i32;

#[derive(Identifiable, Queryable, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Menu {
    pub id: MenuId,
    pub title: String,
    pub slug: MenuSlug,
    pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(feature = "meilisearch-sdk")]
impl meilisearch_sdk::document::Document for Menu {
    type UIDType = MenuId;

    fn get_uid(&self) -> &Self::UIDType {
        &self.id
    }
}

#[cfg(feature = "meilisearch-sdk")]
impl crate::MeiliIndexable for Menu {
    const MEILI_INDEX: &'static str = "menus";
}

#[derive(Debug, Insertable)]
#[table_name = "menus"]
pub struct NewMenu {
    pub slug: MenuSlug,
    pub title: String,
}

impl From<munin_lib::types::menu::Menu> for NewMenu {
    fn from(menu: munin_lib::types::menu::Menu) -> Self {
        Self {
            slug: menu.slug,
            title: menu.title,
        }
    }
}
