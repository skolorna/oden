mod mpi;
mod skolmaten;
mod sodexo;

use std::str::FromStr;

use chrono::NaiveDate;
use serde::{de, Deserialize, Deserializer, Serialize};

use super::{day::Day, Menu};

use crate::errors::Result;

/// A provider of menus.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Provider {
    Skolmaten,
    Sodexo,
    MPI,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
}

impl Provider {
    pub fn id(&self) -> String {
        match *self {
            Provider::Skolmaten => "skolmaten",
            Provider::Sodexo => "sodexo",
            Provider::MPI => "mpi",
        }
        .to_owned()
    }

    pub fn name(&self) -> String {
        match *self {
            Provider::Skolmaten => "Skolmaten",
            Provider::Sodexo => "Sodexo",
            Provider::MPI => "MPI",
        }
        .to_owned()
    }

    pub fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: self.name(),
            id: self.id(),
        }
    }

    pub async fn list_menus(&self) -> Result<Vec<Menu>> {
        use Provider::*;

        match *self {
            Skolmaten => skolmaten::list_menus().await,
            Sodexo => sodexo::list_menus().await,
            MPI => mpi::list_menus().await,
        }
    }

    pub async fn list_all_menus() -> Result<Vec<Menu>> {
        use Provider::*;

        let mut menus = vec![];

        let mut skolmaten_menus = Skolmaten.list_menus().await?;
        let mut sodexo_menus = Sodexo.list_menus().await?;
        let mut mpi_menus = MPI.list_menus().await?;

        menus.append(&mut skolmaten_menus);
        menus.append(&mut sodexo_menus);
        menus.append(&mut mpi_menus);

        Ok(menus)
    }

    pub async fn query_menu(&self, menu_id: &str) -> Result<Menu> {
        use Provider::*;

        match *self {
            Skolmaten => skolmaten::query_menu(menu_id).await,
            Sodexo => sodexo::query_menu(menu_id).await,
            MPI => mpi::query_menu(menu_id).await,
        }
    }

    pub async fn list_days(
        &self,
        menu_id: &str,
        first: NaiveDate,
        last: NaiveDate,
    ) -> Result<Vec<Day>> {
        use Provider::*;

        match *self {
            Skolmaten => skolmaten::list_days(menu_id, first, last).await,
            Sodexo => sodexo::list_days(menu_id, first, last).await,
            MPI => mpi::list_days(menu_id, first, last).await,
        }
    }
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
            "sodexo" => Ok(Provider::Sodexo),
            "mpi" => Ok(Provider::MPI),
            _ => Err(ParseProviderError::InvalidLiteral),
        }
    }
}

impl ToString for Provider {
    fn to_string(&self) -> String {
        self.id()
    }
}

impl Serialize for Provider {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Provider {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_from_str() {
        let s = Provider::Skolmaten.to_string();
        assert_eq!(s, "skolmaten");
        assert_eq!(Provider::from_str(&s).unwrap(), Provider::Skolmaten);
        assert!(Provider::from_str("skolmat").is_err());
    }

    #[test]
    fn ser_de() {
        let s = serde_json::to_string(&Provider::Skolmaten).unwrap();
        assert_eq!(s, "\"skolmaten\"");
        assert_eq!(
            serde_json::from_str::<Provider>(&s).unwrap(),
            Provider::Skolmaten
        );
        assert!(serde_json::from_str::<Provider>("\"bruh\"").is_err());
    }
}
