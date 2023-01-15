use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "db", derive(sqlx::Encode, sqlx::Decode))]
pub struct Meal(pub String);

#[cfg(feature = "db")]
impl sqlx::Type<sqlx::Postgres> for Meal {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

#[cfg(feature = "db")]
impl sqlx::postgres::PgHasArrayType for Meal {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::postgres::PgHasArrayType>::array_type_info()
    }
}

#[derive(Debug)]
pub struct ParseError;

impl FromStr for Meal {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
            Err(ParseError)
        } else {
            Ok(Self(value))
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
            Meal::from_str("              Fisk Björkeby ").unwrap().0,
            "Fisk Björkeby"
        );
        assert_eq!(
            Meal::from_str("Fisk\t\t          Björkeby med ris     \n")
                .unwrap()
                .0,
            "Fisk Björkeby med ris"
        );
    }
}
