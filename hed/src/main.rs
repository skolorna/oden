use std::{convert::Infallible, path::Path, collections::HashSet};

use clap::Parser;
use futures_util::{stream, TryStreamExt, StreamExt};
use hed::archive;
use serde::{Deserialize, Deserializer};
use time::Date;
use tokio::{fs::File, io::BufReader};
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

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    Import,
    Export,
}

async fn import(path: &Path) -> anyhow::Result<()> {
    let mut rdr = csv::ReaderBuilder::new().from_path(path)?;
    let mut records = rdr
        .deserialize::<CsvDay>()
        .flat_map(|r| {
            let d: hed::Day = r.unwrap().into();

            d.meals
                .into_iter()
                .enumerate()
                .map(move |(i, value)| hed::archive::Record {
                    key: hed::meal::Key {
                        menu: d.menu,
                        date: d.date,
                        i,
                    },
                    value,
                })
        })
        .collect::<Vec<hed::archive::Record>>();
        println!("deserialized {} records", records.len());
        
        records.sort_by_key(|r| r.key);
        records.dedup(); // postgres returns strange data
        println!("sorted");

    let wtr = File::create("./data").await?;

    archive::write(wtr, stream::iter(records.into_iter().map(Ok::<_, Infallible>))).await?;

    Ok(())
}

async fn export(archive: &Path) -> anyhow::Result<()> {
    let file = File::open(archive).await?;
    let mut records = archive::read(BufReader::new(file));
    let mut n = 0;

    while let Some(record) = records.try_next().await? {
        // println!("{}: {}", record.key, record.value);
        n+=1;
    }

    println!("{n}");

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Import => import("../supa/days.csv".as_ref()).await,
        Command::Export => export("./data".as_ref()).await,
    }
}
