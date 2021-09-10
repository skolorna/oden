mod kleins;
mod mpi;
mod sabis;
mod skolmaten;
mod sodexo;

use std::str::FromStr;

use chrono::NaiveDate;
use serde::{de, Deserialize, Deserializer, Serialize};
use strum::{EnumIter, EnumString, IntoEnumIterator};

use super::{day::Day, Menu};

use crate::errors::Result;

/// A provider of menus.
#[derive(PartialEq, Debug, Clone, Copy, EnumString, strum::ToString, EnumIter)]
#[strum(serialize_all = "lowercase")]
pub enum Provider {
    Skolmaten,
    Sodexo,
    MPI,
    Kleins,
    Sabis,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
}

impl Provider {
    pub fn id(&self) -> String {
        self.to_string()
    }

    pub fn name(&self) -> String {
        match *self {
            Provider::Skolmaten => "Skolmaten",
            Provider::Sodexo => "Sodexo",
            Provider::MPI => "MPI",
            Provider::Kleins => "Klein's Kitchen",
            Provider::Sabis => "Sabis",
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
            Kleins => kleins::list_menus().await,
            Sabis => sabis::list_menus().await,
        }
    }

    pub async fn list_all_menus() -> Result<Vec<Menu>> {
        let mut menus = vec![];

        for p in Self::iter() {
            menus.append(&mut p.list_menus().await?);
        }

        menus.sort_by(|a, b| a.title.cmp(&b.title));

        Ok(menus)
    }

    pub async fn query_menu(&self, menu_id: &str) -> Result<Menu> {
        use Provider::*;

        match *self {
            Skolmaten => skolmaten::query_menu(menu_id).await,
            Sodexo => sodexo::query_menu(menu_id).await,
            MPI => mpi::query_menu(menu_id).await,
            Kleins => kleins::query_menu(menu_id).await,
            Sabis => todo!(),
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
            Kleins => kleins::list_days(menu_id, first, last).await,
            Sabis => todo!(),
        }
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

    #[actix_rt::test]
    async fn sodexo_query_menu() {
        assert_eq!(
            Provider::Sodexo
                .query_menu("10910e60-20ca-4478-b864-abd8007ad970")
                .await
                .unwrap()
                .title,
            "Södermalmsskolan"
        );
        assert!(Provider::Sodexo.query_menu("bruh").await.is_err());
    }

    #[actix_rt::test]
    async fn kleins_query_menu() {
        let menu = Provider::Kleins
            .query_menu("viktor-rydberg-grundskola-jarlaplan")
            .await
            .unwrap();
        assert_eq!(menu.title, "Viktor Rydberg Gymnasium Jarlaplan");
        assert!(Provider::Kleins.query_menu("nonexistent").await.is_err());
    }
}
