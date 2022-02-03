use chrono::Duration;
use chrono::TimeZone;
use chrono::Utc;
use chrono_tz::Europe::Stockholm;
use database::models::day::NewDay;
use database::models::menu::Menu;
use database::models::menu::MenuId;
use database::models::menu::NewMenu;
use database::MeiliIndexable;
use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use diesel::PgConnection;
use futures::stream;
use futures::StreamExt;
use meilisearch_sdk::client::Client;
use munin_lib::errors::MuninResult;
use munin_lib::menus::list_days;
use munin_lib::menus::list_menus;
use munin_lib::types;
use munin_lib::types::slug::MenuSlug;
use structopt::StructOpt;
use thiserror::Error;
use tracing::info;
use tracing::warn;

#[derive(Debug, StructOpt)]
struct MeiliOpt {}

#[derive(Debug, StructOpt)]
#[structopt(about = "Download menu data for upcoming days")]
pub struct IndexerOpt {
    /// Download new menus and insert them, if not already present.
    #[structopt(long)]
    load_menus: bool,

    /// How many days to fetch for each menu
    #[structopt(long, default_value = "90")]
    days: u32,

    #[structopt(long, default_value = "50")]
    concurrent: usize,

    /// Download the data for a few menus at a time in order to limit
    /// memory usage.
    #[structopt(long, default_value = "500")]
    menus_per_chunk: usize,

    #[structopt(long, short = "l")]
    menu_limit: Option<i64>,

    /// All menus that were updated earlier than this will be selected.
    #[structopt(long, default_value = "86400")]
    max_age_secs: i64,

    /// If provided, the menus will be inserted into the given
    /// MeiliSearch instance.
    #[structopt(long, env)]
    meili_url: Option<String>,

    #[structopt(long, env, hide_env_values = true, default_value = "")]
    meili_key: String,

    #[structopt(long, default_value = Menu::MEILI_INDEX)]
    meili_index: String,
}

pub async fn index(connection: &PgConnection, opt: &IndexerOpt) -> IndexerResult<()> {
    if opt.load_menus {
        load_menus(connection).await?;
    }

    let must_update = get_candidates(
        connection,
        Duration::seconds(opt.max_age_secs),
        opt.menu_limit,
    )?;

    info!("Updating {} menus ...", must_update.len());

    let utc = Utc::now().naive_utc().date();
    let first = Stockholm.from_utc_date(&utc).naive_local();
    let last = first + Duration::days(opt.days as i64);

    let mut stream = stream::iter(must_update)
        .map(|(id, slug)| async move {
            let res = list_days(&slug, first, last).await;
            (id, res)
        })
        .buffer_unordered(opt.concurrent)
        .chunks(opt.menus_per_chunk);

    while let Some(chunk) = stream.next().await {
        submit_days(connection, chunk)?;
    }

    if let Some(meili_url) = &opt.meili_url {
        use database::schema::menus::dsl::*;

        let client = Client::new(meili_url, &opt.meili_key);
        let index = client.get_or_create(&opt.meili_index).await?;

        let documents: Vec<Menu> = menus.load(connection)?;

        let progress = index.add_documents(&documents, None).await?;

        info!(
            "Queued {} documents for MeiliSearch indexing",
            documents.len()
        );

        match progress.wait_for_pending_update(None, None).await {
            Some(Ok(meilisearch_sdk::progress::UpdateStatus::Processed { content })) => {
                info!(
                    "Indexed {} documents in {} seconds",
                    documents.len(),
                    content.duration
                );

                Ok(())
            }
            Some(Err(e)) => Err(e.into()),
            _ => Err(IndexerError::Timeout {
                action: "documents to be indexed".into(),
            }),
        }
    } else {
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("{0}")]
    DieselError(#[from] diesel::result::Error),

    #[error("{0}")]
    MuninError(#[from] munin_lib::errors::MuninError),

    #[error("{0}")]
    MeiliError(#[from] meilisearch_sdk::errors::Error),

    #[error("timeout waiting for {action}")]
    Timeout { action: String },
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
