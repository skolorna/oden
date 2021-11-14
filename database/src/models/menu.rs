use chrono::{DateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable};
use munin_lib::types::slug::MenuSlug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::menus;

pub type MenuId = Uuid;

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
    pub id: Uuid,
    pub slug: MenuSlug,
    pub title: String,
}

impl From<munin_lib::types::menu::Menu> for NewMenu {
    fn from(menu: munin_lib::types::menu::Menu) -> Self {
        Self {
            id: menu.get_uuid(),
            slug: menu.slug,
            title: menu.title,
        }
    }
}
