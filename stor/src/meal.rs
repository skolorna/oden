use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "db", derive(minicbor::Encode, minicbor::Decode))]
pub struct Meal {
    #[cfg_attr(feature = "db", n(0))]
    pub value: String,
}

#[derive(Debug)]
pub struct ParseError;

impl FromStr for Meal {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s
            .trim()
            .split_whitespace()
            .fold(String::new(), |mut result, word| {
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str(word);
                result
            });

        if value.is_empty() {
            Err(ParseError)
        } else {
            Ok(Self { value })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::Meal;

    #[test]
    fn meal() {
        assert!(Meal::from_str("      \t\n    ").is_err());
        assert_eq!(
            Meal::from_str("              Fisk Björkeby ")
                .unwrap()
                .value,
            "Fisk Björkeby"
        );
        assert_eq!(
            Meal::from_str("Fisk\t\t          Björkeby med ris     \n")
                .unwrap()
                .value,
            "Fisk Björkeby med ris"
        );
    }
}
