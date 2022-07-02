mod kleins;
mod matilda;
mod mpi;
mod sabis;
mod skolmaten;
mod sodexo;

use std::str::FromStr;

use chrono::NaiveDate;
use serde::{de, Deserialize, Deserializer, Serialize};
use strum::{EnumIter, EnumString};
use tracing::{debug, instrument};

use crate::{
    errors::{Error, Result},
    menu::Menu,
    Day,
};

/// A provider of menus.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, EnumString, strum::Display, EnumIter)]
#[strum(serialize_all = "lowercase")]
pub enum Supplier {
    Skolmaten,
    Sodexo,
    MPI,
    Kleins,
    Sabis,
    Matilda,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Info {
    pub id: String,
    pub name: String,
}

impl Supplier {
    #[must_use]
    pub fn id(&self) -> String {
        self.to_string()
    }

    #[must_use]
    pub fn name(&self) -> String {
        match *self {
            Supplier::Skolmaten => "Skolmaten",
            Supplier::Sodexo => "Sodexo",
            Supplier::MPI => "MPI",
            Supplier::Kleins => "Klein's Kitchen",
            Supplier::Sabis => "Sabis",
            Supplier::Matilda => "Matilda",
        }
        .to_owned()
    }

    #[must_use]
    pub fn info(&self) -> Info {
        Info {
            name: self.name(),
            id: self.id(),
        }
    }

    #[instrument(err)]
    pub async fn list_menus(&self) -> Result<Vec<Menu>> {
        use Supplier::{Kleins, Matilda, Sabis, Skolmaten, Sodexo, MPI};

        debug!("listing menus");

        match *self {
            Skolmaten => skolmaten::list_menus().await,
            Sodexo => sodexo::list_menus().await,
            MPI => mpi::list_menus().await,
            Kleins => kleins::list_menus().await,
            Sabis => sabis::list_menus().await,
            Matilda => matilda::list_menus().await,
        }
    }

    #[instrument(err)]
    pub async fn list_days(
        &self,
        menu_slug: &str,
        first: NaiveDate,
        last: NaiveDate,
    ) -> Result<Vec<Day>> {
        use Supplier::{Kleins, Matilda, Sabis, Skolmaten, Sodexo, MPI};

        debug!("listing days");

        match *self {
            Skolmaten => {
                skolmaten::list_days(
                    menu_slug.parse().map_err(|_| Error::InvalidMenuSlug)?,
                    first,
                    last,
                )
                .await
            }
            Sodexo => sodexo::list_days(menu_slug, first, last).await,
            MPI => mpi::list_days(menu_slug, first, last).await,
            Kleins => kleins::list_days(menu_slug, first, last).await,
            Sabis => sabis::list_days(menu_slug, first, last).await,
            Matilda => {
                matilda::list_days(
                    &menu_slug.parse().map_err(|_| Error::InvalidMenuSlug)?,
                    first,
                    last,
                )
                .await
            }
        }
    }
}

impl Serialize for Supplier {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Supplier {
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
        let s = Supplier::Skolmaten.to_string();
        assert_eq!(s, "skolmaten");
        assert_eq!(Supplier::from_str(&s).unwrap(), Supplier::Skolmaten);
        assert!(Supplier::from_str("skolmat").is_err());
    }

    #[test]
    fn ser_de() {
        let s = serde_json::to_string(&Supplier::Skolmaten).unwrap();
        assert_eq!(s, "\"skolmaten\"");
        assert_eq!(
            serde_json::from_str::<Supplier>(&s).unwrap(),
            Supplier::Skolmaten
        );
        assert!(serde_json::from_str::<Supplier>("\"bruh\"").is_err());
    }
}
