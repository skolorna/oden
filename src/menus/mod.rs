pub mod providers;

use std::str::FromStr;

use async_trait::async_trait;
use chrono::NaiveDate;
use serde::Serialize;

use crate::errors;

pub type ProviderID = String;

#[derive(Serialize)]
pub struct LocalMenu {
    id: String,
    title: String,
}

impl LocalMenu {
    pub fn into_menu(self, provider_id: ProviderID) -> Menu {
        let id = MenuID::new(&provider_id, &self.id);

        Menu {
            id,
            title: self.title,
        }
    }
}

#[derive(Serialize)]
pub struct LocalMeal {
    value: String,
}

#[derive(Serialize)]
pub struct LocalDay {
    meals: Vec<LocalMeal>,
    /// Time zones aren't really relevant here.
    date: NaiveDate,
}

#[derive(Serialize)]
pub struct ProviderInfo {
    pub name: String,
    pub id: ProviderID,
}

#[async_trait]
pub trait Provider {
    fn id() -> ProviderID;

    fn name() -> String;

    fn info() -> ProviderInfo {
        ProviderInfo {
            name: Self::id(),
            id: Self::id(),
        }
    }

    async fn list_menus() -> errors::Result<Vec<LocalMenu>>;

    async fn query_menu(menu_id: &str) -> errors::Result<LocalMenu>;

    async fn list_days(menu_id: &str) -> errors::Result<Vec<LocalDay>>;
}

#[derive(PartialEq, Debug)]
pub struct MenuID {
    pub local_id: String,
    pub provider_id: ProviderID,
}

impl MenuID {
    pub fn new(provider_id: &str, local_id: &str) -> Self {
        Self {
            provider_id: provider_id.to_owned(),
            local_id: local_id.to_owned(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseMenuIDError {
    #[error("id delimiter missing")]
    NoDelimiter,

    #[error("fields missing")]
    FieldsMissing,
}

impl FromStr for MenuID {
    type Err = ParseMenuIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (provider_id, local_id) = s.split_once(".").ok_or(ParseMenuIDError::NoDelimiter)?;

        if provider_id.is_empty() || local_id.is_empty() {
            Err(ParseMenuIDError::FieldsMissing)
        } else {
            Ok(Self::new(provider_id, local_id))
        }
    }
}

impl ToString for MenuID {
    fn to_string(&self) -> String {
        format!("{}.{}", self.provider_id, self.local_id)
    }
}

impl Serialize for MenuID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Serialize)]
pub struct Menu {
    id: MenuID,
    title: String,
}

impl Menu {
    pub fn new(id: MenuID, title: &str) -> Self {
        Self {
            id,
            title: title.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_menu_id() {
        let parsed = MenuID::from_str("lovely_provider.aaa-bbb-ccc").unwrap();

        assert_eq!(parsed.local_id, "aaa-bbb-ccc");
        assert_eq!(parsed.provider_id, "lovely_provider");

        assert!(MenuID::from_str("invalid").is_err());
        assert!(MenuID::from_str(".").is_err());
        assert!(MenuID::from_str("a.").is_err());
        assert!(MenuID::from_str(".a").is_err());
    }

    #[test]
    fn menu_id_eq() {
        let a = MenuID::new("provider-a", "local-a");
        let b = MenuID::new("provider-a", "local-b");
        assert_ne!(a, b);
        let c = MenuID::new("provider-a", "local-a");
        assert_eq!(a, c);
        let d = MenuID::new("provider-b", "local-a");
        assert_ne!(a, d);
    }

    #[test]
    fn menu_id_roundtrip() {
        let original = MenuID::new("provider", "local-id");
        let serialized = original.to_string();
        assert_eq!(serialized, "provider.local-id");
        let parsed = MenuID::from_str(&serialized).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn menu_id_serde() {
        let id = MenuID::new("provider", "local");
        let str = serde_json::to_string(&id).unwrap();
        assert_eq!(str, "\"provider.local\"");
    }

    #[test]
    fn local_menu_to_menu() {
        let local_menu = LocalMenu {
            title: "Menu".to_owned(),
            id: "abc123".to_owned(),
        };

        assert_eq!(
            local_menu.into_menu("provider-1".to_string()).id,
            MenuID::new("provider-1", "abc123")
        );
    }
}
