use serde::{Deserialize, Serialize};
use time::Date;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Meal {
    pub menu_id: Uuid,
    pub meal: String,
    pub date: Date,
}

pub fn sanitize_meal_value(s: &str) -> Option<String> {
    let value = s
        .split_whitespace()
        .fold(String::new(), |mut result, word| {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(word);
            result
        });

    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::meal::sanitize_meal_value;

    #[test]
    fn meal() {
        assert!(sanitize_meal_value("      \t\n    ").is_none());
        assert_eq!(
            sanitize_meal_value("              Fisk Björkeby ").unwrap(),
            "Fisk Björkeby"
        );
        assert_eq!(
            sanitize_meal_value("Fisk\t\t          Björkeby med ris     \n").unwrap(),
            "Fisk Björkeby med ris"
        );
    }
}
