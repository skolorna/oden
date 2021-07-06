use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::errors::Result;

use super::{id::MenuID, Day, Menu};

pub mod skolmaten;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Provider {
    Skolmaten,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseProviderError {
    #[error("invalid provider literal")]
    InvalidLiteral,
}

impl FromStr for Provider {
    type Err = ParseProviderError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s {
            "skolmaten" => Ok(Provider::Skolmaten),
            _ => Err(ParseProviderError::InvalidLiteral),
        }
    }
}

impl ToString for Provider {
    fn to_string(&self) -> String {
        match self {
            Provider::Skolmaten => "skolmaten",
        }
        .to_owned()
    }
}

pub async fn list_menus() -> Result<Vec<Menu>> {
    let menus = skolmaten::list_menus().await?;

    Ok(menus)
}

pub async fn query_menu(menu_id: &MenuID) -> Result<Menu> {
    match menu_id.provider {
        Provider::Skolmaten => skolmaten::query_menu(&menu_id.local_id).await,
    }
}

pub async fn list_days(menu_id: &MenuID) -> Result<Vec<Day>> {
    match menu_id.provider {
        Provider::Skolmaten => skolmaten::list_days(&menu_id.local_id).await,
    }
}
