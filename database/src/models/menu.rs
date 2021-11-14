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

pub const UUID_NAMESPACE: Uuid = Uuid::from_bytes([
    0x88, 0xdc, 0x80, 0xe5, 0xf4, 0x7f, 0x46, 0x34, 0xb6, 0x33, 0x2c, 0xce, 0x5e, 0xf2, 0xcb, 0x11,
]);

impl From<munin_lib::types::menu::Menu> for NewMenu {
    fn from(menu: munin_lib::types::menu::Menu) -> Self {
        Self {
            id: Uuid::new_v5(&UUID_NAMESPACE, menu.slug.to_string().as_bytes()),
            slug: menu.slug,
            title: menu.title,
        }
    }
}
