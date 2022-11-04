pub mod kleins;
pub mod matilda;
pub mod mpi;
pub mod sabis;
pub mod skolmaten;
pub mod sodexo;

use std::str::FromStr;

use chrono::NaiveDate;
use reqwest::Client;
use serde::{de, Deserialize, Deserializer, Serialize};
use strum::{EnumIter, EnumString};
use tracing::{debug, instrument};

use crate::{
    errors::{Error, Result},
    menu::Menu,
    Day,
};

/// A supplier of menus.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, EnumString, strum::Display, EnumIter)]
#[strum(serialize_all = "lowercase")]
pub enum Supplier {
    Skolmaten,
    Sodexo,
    Mpi,
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
            Supplier::Mpi => "MPI",
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

    #[instrument(err, skip(client))]
    pub async fn list_menus(&self, client: &Client) -> Result<Vec<Menu>> {
        use Supplier::{Kleins, Matilda, Mpi, Sabis, Skolmaten, Sodexo};

        debug!("listing menus");

        match *self {
            Skolmaten => skolmaten::list_menus(client).await,
            Sodexo => sodexo::list_menus(client).await,
            Mpi => mpi::list_menus(client).await,
            Kleins => kleins::list_menus(client).await,
            Sabis => sabis::list_menus().await,
            Matilda => matilda::list_menus(client).await,
        }
    }

    #[instrument(err, skip(client))]
    pub async fn list_days(
        &self,
        client: &Client,
        menu_slug: &str,
        first: NaiveDate,
        last: NaiveDate,
    ) -> Result<Vec<Day>> {
        use Supplier::{Kleins, Matilda, Mpi, Sabis, Skolmaten, Sodexo};

        debug!("listing days");

        match *self {
            Skolmaten => {
                skolmaten::list_days(
                    client,
                    menu_slug.parse().map_err(|_| Error::InvalidMenuSlug)?,
                    first,
                    last,
                )
                .await
            }
            Sodexo => sodexo::list_days(client, menu_slug, first, last).await,
            Mpi => mpi::list_days(client, menu_slug, first, last).await,
            Kleins => kleins::list_days(client, menu_slug, first, last).await,
            Sabis => sabis::list_days(client, menu_slug, first, last).await,
            Matilda => {
                matilda::list_days(
                    client,
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
