use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize};

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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Provider {
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
