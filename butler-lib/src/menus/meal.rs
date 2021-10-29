use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum ParseMealValueError {
    #[error("invalid meal value")]
    InvalidMealValue,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Meal {
    value: String,
}

impl Meal {
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl FromStr for Meal {
    type Err = ParseMealValueError;

    /// Construct a `Meal` with a specific value. The value will be normalized.
    /// Some meal values are considered invalid and will result in `None` being
    /// returned.
    /// ```
    /// use butler::menus::meal::Meal;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Meal::from_str("\t  Fisk Björkeby   \n").unwrap().value(), "Fisk Björkeby");
    /// assert!(Meal::from_str("\n\n\n").is_err());
    /// ```
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed = value.split_whitespace().collect::<Vec<&str>>().join(" ");

        if trimmed.is_empty() || trimmed.to_lowercase().contains("lov") {
            Err(ParseMealValueError::InvalidMealValue)
        } else {
            Ok(Self { value: trimmed })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(Meal::from_str("\t\n  dET ÄR SommarLOV!!!!!\n\n").is_err());
    }
}
