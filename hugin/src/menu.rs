#![allow(clippy::module_name_repetitions)]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod slug;

pub use slug::Slug as MenuSlug;

#[derive(Debug, Serialize, Deserialize)]
pub struct Menu {
    pub slug: MenuSlug,
    pub title: String,
}

impl Menu {
    pub const UUID_NAMESPACE: Uuid = Uuid::from_bytes([
        0x88, 0xdc, 0x80, 0xe5, 0xf4, 0x7f, 0x46, 0x34, 0xb6, 0x33, 0x2c, 0xce, 0x5e, 0xf2, 0xcb,
        0x11,
    ]);

    #[must_use]
    pub fn new(slug: MenuSlug, title: impl Into<String>) -> Self {
        Self {
            slug,
            title: title.into(),
        }
    }

    #[must_use]
    pub fn slug(&self) -> &MenuSlug {
        &self.slug
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn get_uuid(&self) -> Uuid {
        Uuid::new_v5(&Self::UUID_NAMESPACE, self.slug.to_string().as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use crate::{menus::supplier::Supplier, MenuSlug};

    use super::Menu;

    #[test]
    fn uuid_gen() {
        let slug = MenuSlug::new(Supplier::Sodexo, "skool");

        let a = Menu::new(slug.clone(), "Skool 1");
        let b = Menu::new(slug, "Rebranded skool");

        assert_eq!(a.get_uuid(), b.get_uuid());
    }
}
