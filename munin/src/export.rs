use std::{
    convert::{TryFrom, TryInto},
    fs::{create_dir, OpenOptions},
    path::PathBuf,
};

use chrono::NaiveDate;
use database::models::{Day, Menu, MenuId};
use database::schema::{days::table as days_table, menus::table as menus_table};
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use hugin::Meal;
use serde::Serialize;
use tracing::info;

#[derive(Debug, Serialize)]
struct ExportedDay {
    menu_id: MenuId,
    date: NaiveDate,
    meals: String,
}

impl TryFrom<Day> for ExportedDay {
    type Error = serde_json::Error;

    fn try_from(d: Day) -> Result<Self, Self::Error> {
        let Day {
            date,
            meals,
            menu_id,
        } = d;
        let meals: Vec<Meal> = meals.into();

        Ok(Self {
            menu_id,
            date,
            meals: serde_json::to_string(&meals)?,
        })
    }
}

pub fn days_chunks<F>(
    connection: &PgConnection,
    chunk_size: i64,
    mut on_chunk: F,
) -> anyhow::Result<()>
where
    F: FnMut(Vec<Day>),
{
    connection
        .build_transaction()
        .repeatable_read()
        .read_only()
        .run::<_, anyhow::Error, _>(|| {
            let mut offset = 0;

            loop {
                let days = days_table
                    .offset(offset)
                    .limit(chunk_size)
                    .load::<Day>(connection)?;
                let num_days = days.len();

                info!("loaded {} days", num_days);

                (on_chunk)(days);

                if (num_days as i64) < chunk_size {
                    break;
                }

                offset += chunk_size;
            }

            Ok(())
        })?;

    Ok(())
}

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(short, long)]
    output: PathBuf,

    #[arg(long, default_value = "100000")]
    chunk_size: i64,
}

pub fn export(connection: &PgConnection, opt: &Args) -> anyhow::Result<()> {
    create_dir(&opt.output)?;

    let menus_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(opt.output.join("menus.csv"))?;
    let mut menus_w = csv::Writer::from_writer(menus_file);

    let days_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(opt.output.join("days.csv"))?;
    let mut days_w = csv::Writer::from_writer(days_file);

    connection
        .build_transaction()
        .repeatable_read()
        .read_only()
        .run::<_, anyhow::Error, _>(|| {
            let menus = menus_table.load::<Menu>(connection)?;

            for menu in menus {
                menus_w.serialize(menu)?;
            }

            let mut offset = 0;

            loop {
                let days = days_table
                    .offset(offset)
                    .limit(opt.chunk_size)
                    .load::<Day>(connection)?;
                let num_days = days.len();

                info!("loaded {} days", num_days);

                for day in days {
                    let day: ExportedDay = day.try_into()?;
                    days_w.serialize(day)?;
                }

                if (num_days as i64) < opt.chunk_size {
                    break;
                }

                offset += opt.chunk_size;
            }

            Ok(())
        })?;

    menus_w.flush()?;
    days_w.flush()?;

    Ok(())
}
