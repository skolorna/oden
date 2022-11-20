use std::{fs, path::Path, thread::sleep, time::Duration};

use serde::{Deserialize, Deserializer};
use time::Date;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Meal {
    value: String,
}

fn deserialize_meals<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let meals: Vec<Meal> = serde_json::from_str(&s).map_err(serde::de::Error::custom)?;
    Ok(meals.into_iter().map(|m| m.value).collect())
}

#[derive(Debug, Deserialize)]
struct CsvDay {
    #[serde(deserialize_with = "deserialize_meals")]
    meals: Vec<String>,
    date: Date,
    menu_id: Uuid,
}

impl From<CsvDay> for hed::Day {
    fn from(d: CsvDay) -> Self {
        Self {
            menu: d.menu_id,
            date: d.date,
            meals: d.meals,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut rdr = csv::ReaderBuilder::new().from_path("../supa/days.csv")?;
    let records = rdr
        .deserialize::<CsvDay>()
        .map(|r| r.map(Into::into))
        .collect::<Result<Vec<hed::Day>, _>>()?;
    println!("deserialized {} records", records.len());

    for day in records {}

    sleep(Duration::from_secs(6000000000));

    Ok(())
}
