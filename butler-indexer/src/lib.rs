use butler_lib::errors::ButlerResult;
use butler_lib::menus::id::MenuId;
use butler_lib::menus::list_menus;
use butler_lib::types;
use chrono::Duration;
use chrono::Utc;
use database::models::day::NewDay;
use database::models::menu::NewMenu;
use diesel::prelude::*;
use diesel::PgConnection;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("{0}")]
    DieselError(#[from] diesel::result::Error),

    #[error("{0}")]
    ButlerError(#[from] butler_lib::errors::ButlerError),
}

pub type IndexerResult<T> = Result<T, IndexerError>;

/// Indexes the menus from the suppliers, and stores them in the database. If
/// the menu already exists, it won't be updated. Returns the number of menus
/// inserted.
pub async fn load_menus(connection: &PgConnection) -> IndexerResult<usize> {
    use database::schema::menus::dsl::*;

    let records = list_menus(4)
        .await?
        .into_iter()
        .map(|m| m.into())
        .collect::<Vec<NewMenu>>();

    info!("Fetched {} menus", records.len());

    let inserted_count = diesel::insert_into(menus)
        .values(&records)
        .on_conflict_do_nothing()
        .execute(connection)?;

    match inserted_count {
        0 => info!("No new menus were inserted"),
        _ => info!("Inserted {} new menus", inserted_count),
    }

    Ok(inserted_count)
}

pub fn get_candidates(connection: &PgConnection, limit: Option<i64>) -> QueryResult<Vec<MenuId>> {
    use database::schema::menus::dsl::*;

    let q = menus.select(id).filter(
        updated_at
            .lt(Utc::now() - Duration::days(1))
            .or(updated_at.is_null()),
    );

    if let Some(limit) = limit {
        q.limit(limit).load(connection)
    } else {
        q.load::<MenuId>(connection)
    }
}

pub fn insert_days_and_update_menus(
    connection: &PgConnection,
    results: Vec<(MenuId, ButlerResult<Vec<types::day::Day>>)>,
) -> QueryResult<()> {
    use database::schema::days::table as days_table;
    use database::schema::menus::{columns as menus_columns, table as menus_table};

    let successful = results
        .iter()
        .filter_map(|(m, r)| r.as_ref().ok().map(|_| m.clone()))
        .collect::<Vec<_>>();

    let records = results
        .into_iter()
        .filter_map(|(m, r)| {
            let d = r
                .ok()?
                .into_iter()
                .map(|d| NewDay::from_day(d, m.to_owned()))
                .collect::<Vec<_>>();

            Some(d)
        })
        .flatten()
        .collect::<Vec<NewDay>>();

    info!("Inserting {} days", records.len());

    for chunk in records.chunks(10000) {
        diesel::insert_into(days_table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(connection)?;
    }

    let updated_at = Utc::now();

    for chunk in successful.chunks(1000) {
        diesel::update(menus_table.filter(menus_columns::id.eq_any(chunk)))
            .set(menus_columns::updated_at.eq(updated_at))
            .execute(connection)?;
    }

    Ok(())
}
