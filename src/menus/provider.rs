use std::str::FromStr;

use serde::{Deserialize, Serialize};

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
