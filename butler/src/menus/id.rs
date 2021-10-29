use std::{fmt::Display, str::FromStr};

use serde::{de, Deserialize, Deserializer, Serialize};

use super::supplier::Supplier;

#[derive(PartialEq, Debug, Clone)]
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
}

impl FromStr for MenuId {
    type Err = ParseMenuIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
        write!(f, "{}.{}", self.supplier, self.local_id)
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
