use serde::Serialize;

#[derive(Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
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
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return None;
        }

        Some(Self {
            value: trimmed.to_owned(),
        })
    }
}
