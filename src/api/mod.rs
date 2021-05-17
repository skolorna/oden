pub mod skolmaten;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Meal {
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Menu {
    pub timestamp: DateTime<Utc>,
    pub meals: Vec<Meal>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct School {
    pub id: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_serde() {
        let json_str = r#"
          {
            "timestamp": "2017-02-16T00:00:00Z",
            "meals": [{
              "value": "Fisk Björkeby"
            }]
          }
        "#;

        let data: Menu = serde_json::from_str(json_str).unwrap();
        assert_eq!(data.timestamp.timestamp(), 1487203200);
        assert_eq!(data.meals.get(0).unwrap().value, "Fisk Björkeby");
        assert_eq!(data.meals.len(), 1);
    }
}
