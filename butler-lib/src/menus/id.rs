use std::{fmt::Display, str::FromStr};

use base64::display::Base64Display;
#[cfg(feature = "diesel")]
use diesel::{
    backend::Backend,
    sql_types,
    types::{FromSql, ToSql},
    AsExpression, FromSqlRow,
};
use serde::{de, Deserialize, Deserializer, Serialize};

use super::supplier::Supplier;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
#[cfg_attr(feature = "diesel", derive(AsExpression, FromSqlRow))]
#[cfg_attr(feature = "diesel", sql_type = "diesel::sql_types::Text")]
pub struct MenuId {
    pub supplier: Supplier,
    pub local_id: String,
}

impl MenuId {
    pub fn new(supplier: Supplier, local_id: String) -> Self {
        Self { supplier, local_id }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseMenuIDError {
    #[error("id delimiter missing")]
    NoDelimiter,

    #[error("fields missing")]
    FieldsMissing,

    #[error("failed to parse supplier name")]
    ParseSupplierError(#[from] strum::ParseError),

    #[error("{0}")]
    Base64Error(#[from] base64::DecodeError),
}

impl FromStr for MenuId {
    type Err = ParseMenuIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = String::from_utf8(base64::decode_config(s, base64::URL_SAFE_NO_PAD)?)
            .map_err(|_| ParseMenuIDError::FieldsMissing)?;

        let (supplier_literal, local_id) =
            s.split_once(".").ok_or(ParseMenuIDError::NoDelimiter)?;

        let supplier = Supplier::from_str(supplier_literal)?;

        if local_id.is_empty() {
            Err(ParseMenuIDError::FieldsMissing)
        } else {
            Ok(Self::new(supplier, local_id.to_owned()))
        }
    }
}

impl Display for MenuId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let raw = format!("{}.{}", self.supplier, self.local_id);
        write!(
            f,
            "{}",
            Base64Display::with_config(raw.as_bytes(), base64::URL_SAFE_NO_PAD)
        )
    }
}

impl Serialize for MenuId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for MenuId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

#[cfg(feature = "diesel")]
impl<DB> ToSql<sql_types::Text, DB> for MenuId
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
impl<DB> FromSql<sql_types::Text, DB> for MenuId
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
    fn parse_menu_id() {
        let parsed = MenuId::from_str("skolmaten.aaa-bbb-ccc").unwrap();

        assert_eq!(parsed.local_id, "aaa-bbb-ccc");
        assert_eq!(parsed.supplier, Supplier::Skolmaten);

        assert!(MenuId::from_str("invalid").is_err());
        assert!(MenuId::from_str(".").is_err());
        assert!(MenuId::from_str("skolmaten.").is_err());
        assert!(MenuId::from_str(".abc").is_err());
    }

    #[test]
    fn menu_id_eq() {
        let a = MenuId::new(Supplier::Skolmaten, "foo".to_owned());
        let b = MenuId::new(Supplier::Skolmaten, "bar".to_owned());
        assert_ne!(a, b);
        let c = MenuId::new(Supplier::Skolmaten, "foo".to_owned());
        assert_eq!(a, c);
    }

    #[test]
    fn menu_id_roundtrip() {
        let original = MenuId::new(Supplier::Skolmaten, "local-id".to_owned());
        let serialized = original.to_string();
        assert_eq!(serialized, "skolmaten.local-id");
        let parsed = MenuId::from_str(&serialized).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn menu_id_ser() {
        let id = MenuId::new(Supplier::Skolmaten, "local".to_owned());
        let s = serde_json::to_string(&id).unwrap();
        assert_eq!(s, "\"skolmaten.local\"");
    }

    #[test]
    fn menu_id_de() {
        let s = "\"skolmaten.local\"";
        assert_eq!(
            serde_json::from_str::<MenuId>(s).unwrap(),
            MenuId::new(Supplier::Skolmaten, "local".to_owned())
        );

        assert!(serde_json::from_str::<MenuId>("\"bruh.local\"").is_err());
    }
}
