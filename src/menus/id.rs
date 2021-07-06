use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize};

use super::provider::{ParseProviderError, Provider};

#[derive(PartialEq, Debug)]
pub struct MenuID {
    pub provider: Provider,
    pub local_id: String,
}

impl MenuID {
    pub fn new(provider: Provider, local_id: String) -> Self {
        Self { provider, local_id }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseMenuIDError {
    #[error("id delimiter missing")]
    NoDelimiter,

    #[error("fields missing")]
    FieldsMissing,

    #[error("{0}")]
    ParseProviderError(#[from] ParseProviderError),
}

impl FromStr for MenuID {
    type Err = ParseMenuIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (provider_literal, local_id) =
            s.split_once(".").ok_or(ParseMenuIDError::NoDelimiter)?;

        let provider = Provider::from_str(provider_literal)?;

        if local_id.is_empty() {
            Err(ParseMenuIDError::FieldsMissing)
        } else {
            Ok(Self::new(provider, local_id.to_owned()))
        }
    }
}

impl ToString for MenuID {
    fn to_string(&self) -> String {
        format!("{}.{}", self.provider.to_string(), self.local_id)
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

impl<'de> Deserialize<'de> for MenuID {
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
        let parsed = MenuID::from_str("skolmaten.aaa-bbb-ccc").unwrap();

        assert_eq!(parsed.local_id, "aaa-bbb-ccc");
        assert_eq!(parsed.provider, Provider::Skolmaten);

        assert!(MenuID::from_str("invalid").is_err());
        assert!(MenuID::from_str(".").is_err());
        assert!(MenuID::from_str("a.").is_err());
        assert!(MenuID::from_str(".a").is_err());
    }

    #[test]
    fn menu_id_eq() {
        let a = MenuID::new(Provider::Skolmaten, "foo".to_owned());
        let b = MenuID::new(Provider::Skolmaten, "bar".to_owned());
        assert_ne!(a, b);
        let c = MenuID::new(Provider::Skolmaten, "foo".to_owned());
        assert_eq!(a, c);
    }

    #[test]
    fn menu_id_roundtrip() {
        let original = MenuID::new(Provider::Skolmaten, "local-id".to_owned());
        let serialized = original.to_string();
        assert_eq!(serialized, "skolmaten.local-id");
        let parsed = MenuID::from_str(&serialized).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn menu_id_ser() {
        let id = MenuID::new(Provider::Skolmaten, "local".to_owned());
        let s = serde_json::to_string(&id).unwrap();
        assert_eq!(s, "\"skolmaten.local\"");
    }

    #[test]
    fn menu_id_de() {
        let s = "\"skolmaten.local\"";
        assert_eq!(
            serde_json::from_str::<MenuID>(s).unwrap(),
            MenuID::new(Provider::Skolmaten, "local".to_owned())
        );

        assert!(serde_json::from_str::<MenuID>("\"bruh.local\"").is_err());
    }
}
