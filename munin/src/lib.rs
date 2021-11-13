use chrono::Duration;
use chrono::Utc;
use database::models::day::NewDay;
use database::models::menu::MenuId;
use database::models::menu::NewMenu;
use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use diesel::PgConnection;
use munin_lib::errors::MuninResult;
use munin_lib::menus::list_menus;
use munin_lib::types;
use munin_lib::types::slug::MenuSlug;
use thiserror::Error;
use tracing::info;
use tracing::warn;

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("{0}")]
    DieselError(#[from] diesel::result::Error),

    #[error("{0}")]
    MuninError(#[from] munin_lib::errors::MuninError),

    #[error("{0}")]
    MeiliError(#[from] meilisearch_sdk::errors::Error),
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

pub fn get_candidates(
    connection: &PgConnection,
    max_age: Duration,
    limit: Option<i64>,
) -> QueryResult<Vec<(MenuId, MenuSlug)>> {
    use database::schema::menus::dsl::*;

    let q = menus
        .select((id, slug))
        .filter(updated_at.lt(Utc::now() - max_age).or(updated_at.is_null()));

    if let Some(limit) = limit {
        q.limit(limit).load(connection)
    } else {
        q.load(connection)
    }
}

pub fn submit_days(
    connection: &PgConnection,
    results: Vec<(MenuId, MuninResult<Vec<types::day::Day>>)>,
) -> QueryResult<()> {
    use database::schema::days::{columns as days_columns, table as days_table};
    use database::schema::menus::{columns as menus_columns, table as menus_table};

    let successful = results
        .iter()
        .filter_map(|(m, r)| r.as_ref().ok().map(|_| *m))
        .collect::<Vec<_>>();

    let failed = results.len() - successful.len();

    if failed > 0 {
        warn!("{} out of {} downloads failed", failed, results.len());
    }

    let records = results
        .into_iter()
        .filter_map(|(m, r)| {
            let d = r
                .ok()?
                .into_iter()
                .map(|d| NewDay::from_day(d, m))
                .collect::<Vec<_>>();

            Some(d)
        })
        .flatten()
        .collect::<Vec<NewDay>>();

    info!("Inserting {} days", records.len());

    for chunk in records.chunks(10000) {
        diesel::insert_into(days_table)
            .values(chunk)
            // .on_conflict_do_nothing()
            .on_conflict((days_columns::menu_id, days_columns::date))
            .do_update()
            .set(days_columns::meals.eq(excluded(days_columns::meals)))
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
