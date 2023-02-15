#[cfg(feature = "db")]
pub mod db;
pub mod meal;
pub mod menu;
pub mod review;

pub use meal::Meal;
pub use menu::Menu;
pub use review::Review;
use serde::{Deserialize, Serialize};
use time::Date;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Day {
    pub date: Date,
    pub meals: Vec<String>,
}

impl Day {
    pub fn new(date: Date, meals: Vec<String>) -> Self {
        Self { date, meals }
    }
}
