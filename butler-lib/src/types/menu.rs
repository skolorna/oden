use serde::{Deserialize, Serialize};

use crate::menus::id::MenuId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Menu {
    pub id: MenuId,
    pub title: String,
}

impl Menu {
    pub fn new(id: MenuId, title: String) -> Self {
        Self { id, title }
    }

    pub fn id(&self) -> &MenuId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}
