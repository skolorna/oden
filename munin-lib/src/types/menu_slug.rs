use std::{fmt::Display, str::FromStr};

#[cfg(feature = "diesel")]
use diesel::{
    backend::Backend,
    sql_types,
    types::{FromSql, ToSql},
    AsExpression, FromSqlRow,
};
use serde::{de, Deserialize, Deserializer, Serialize};

use crate::menus::supplier::Supplier;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
#[cfg_attr(feature = "diesel", derive(AsExpression, FromSqlRow))]
#[cfg_attr(feature = "diesel", sql_type = "diesel::sql_types::Text")]
pub struct MenuSlug {
    pub supplier: Supplier,
    pub local_id: String,
}

impl MenuSlug {
    #[must_use]
    pub fn new(supplier: Supplier, local_id: String) -> Self {
        Self { supplier, local_id }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseMenuSlugError {
    #[error("id delimiter missing")]
    NoDelimiter,

    #[error("fields missing")]
    FieldsMissing,

    #[error("failed to parse supplier name")]
    ParseSupplierError(#[from] strum::ParseError),
}

impl FromStr for MenuSlug {
    type Err = ParseMenuSlugError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (supplier_literal, local_id) =
            s.split_once('.').ok_or(ParseMenuSlugError::NoDelimiter)?;

        let supplier = Supplier::from_str(supplier_literal)?;

        if local_id.is_empty() {
            Err(ParseMenuSlugError::FieldsMissing)
        } else {
            Ok(Self::new(supplier, local_id.to_owned()))
        }
    }
}

impl Display for MenuSlug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.supplier, self.local_id)
    }
}

impl Serialize for MenuSlug {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for MenuSlug {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

#[cfg(feature = "diesel")]
impl<DB> ToSql<sql_types::Text, DB> for MenuSlug
where
    DB: Backend,
    String: ToSql<sql_types::Text, DB>,
{
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, DB>,
    ) -> diesel::serialize::Result {
        self.to_string().to_sql(out)
    }
}

#[cfg(feature = "diesel")]
impl<DB> FromSql<sql_types::Text, DB> for MenuSlug
where
    DB: Backend,
    String: FromSql<sql_types::Text, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
        let id: Self = String::from_sql(bytes)?.parse()?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_slug_eq() {
        let a = MenuSlug::new(Supplier::Skolmaten, "foo".to_owned());
        let b = MenuSlug::new(Supplier::Skolmaten, "bar".to_owned());
        assert_ne!(a, b);
        let c = MenuSlug::new(Supplier::Skolmaten, "foo".to_owned());
        assert_eq!(a, c);
    }

    #[test]
    fn menu_slug_roundtrip() {
        let original = MenuSlug::new(Supplier::Skolmaten, "local-id".to_owned());
        let serialized = original.to_string();
        assert_eq!(serialized, "skolmaten.local-id");
        let parsed = MenuSlug::from_str(&serialized).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn menu_slug_ser() {
        let id = MenuSlug::new(Supplier::Skolmaten, "local".to_owned());
        let s = serde_json::to_string(&id).unwrap();
        assert_eq!(s, "\"skolmaten.local\"");
    }

    #[test]
    fn menu_slug_de() {
        let s = "\"skolmaten.local\"";
        assert_eq!(
            serde_json::from_str::<MenuSlug>(s).unwrap(),
            MenuSlug::new(Supplier::Skolmaten, "local".to_owned())
        );

        assert!(serde_json::from_str::<MenuSlug>("\"bruh\"").is_err());
    }
}
