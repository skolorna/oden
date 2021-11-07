use serde::{Deserialize, Serialize};

use crate::menus::id::MenuSlug;

#[derive(Debug, Serialize, Deserialize)]
pub struct Menu {
    pub slug: MenuSlug,
    pub title: String,
}

impl Menu {
    pub fn new(slug: MenuSlug, title: String) -> Self {
        Self { slug, title }
    }

    pub fn slug(&self) -> &MenuSlug {
        &self.slug
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}
