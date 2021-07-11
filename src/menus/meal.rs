use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Meal {
    pub value: String,
}

impl Meal {
    /// Construct a `Meal` with a specific value. The value will be normalized.
    /// Some meal values are considered invalid and will result in `None` being
    /// returned.
    /// ```
    /// use menu_proxy::menus::meal::Meal;
    ///
    /// assert_eq!(Meal::from_value("\t  Fisk Björkeby   \n").unwrap().value, "Fisk Björkeby");
    ///
    /// assert!(Meal::from_value("\n\n\n").is_none());
    /// ```
    pub fn from_value(value: &str) -> Option<Self> {
        let trimmed = value.split_whitespace().collect::<Vec<&str>>().join(" ");

        if trimmed.is_empty() || trimmed.to_lowercase().contains("lov") {
            None
        } else {
            Some(Self { value: trimmed })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meal() {
        assert!(Meal::from_value("      \t\n    ").is_none());
        assert_eq!(
            Meal::from_value("              Fisk Björkeby ")
                .unwrap()
                .value,
            "Fisk Björkeby"
        );
        assert_eq!(
            Meal::from_value("Fisk\t\t          Björkeby med ris     \n")
                .unwrap()
                .value,
            "Fisk Björkeby med ris"
        );
        assert!(Meal::from_value("\t\n  dET ÄR SommarLOV!!!!!\n\n").is_none());
    }
}
