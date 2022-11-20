use time::Date;
use uuid::Uuid;

pub struct Day {
    pub menu: Uuid,
    pub date: Date,
    pub meals: Vec<String>,
}
