use anyhow::bail;
use chrono::DateTime;
use chrono::Duration;
use chrono::NaiveDate;
use chrono::TimeZone;
use chrono::Utc;
use chrono_tz::Europe::Stockholm;
use database::models::MenuId;
use database::models::NewDay;
use database::models::NewMenu;
use database::{models::Menu, MeiliIndexable};
use diesel::dsl::sql;
use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use diesel::sql_types::Date;
use diesel::PgConnection;
use futures::stream;
use futures::StreamExt;
use hugin::list_days;
use hugin::list_menus;
use hugin::MenuSlug;
use itertools::Itertools;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::tasks::Task;
use serde::Serialize;
use tracing::{info, instrument, warn};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Download new menus and insert them, if not already present.
    #[arg(long)]
    load_menus: bool,

    /// How many days to fetch for each menu
    #[arg(long, default_value = "90")]
    days: u32,

    #[arg(long, default_value = "50")]
    concurrent: usize,

    /// Download the data for a few menus at a time in order to limit
    /// memory usage.
    #[arg(long, default_value = "500")]
    menus_per_chunk: usize,

    #[arg(long, short = 'l')]
    menu_limit: Option<i64>,

    /// All menus that were updated earlier than this will be selected.
    #[arg(long, default_value = "86400")]
    max_age_secs: i64,

    /// If provided, the menus will be inserted into the given
    /// MeiliSearch instance.
    #[arg(long, env)]
    meili_url: Option<String>,

    #[arg(long, env, hide_env_values = true, default_value = "")]
    meili_key: String,

    #[arg(long, default_value = Menu::MEILI_INDEX)]
    meili_index: String,
}

#[derive(Debug, Serialize)]
struct MeiliMenu {
    id: MenuId,
    updated_at: Option<DateTime<Utc>>,
    slug: MenuSlug,
    title: String,
    last_day: Option<NaiveDate>,
}

pub async fn index(connection: &PgConnection, opt: &Args) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    if opt.load_menus {
        load_menus(connection).await?;
    }

    let must_update = get_candidates(
        connection,
        Duration::seconds(opt.max_age_secs),
        opt.menu_limit,
    )?;

    info!("updating {} menus", must_update.len());

    let utc = Utc::now().naive_utc().date();
    let first = Stockholm.from_utc_date(&utc).naive_local();
    let last = first + Duration::days(opt.days as i64);

    let mut stream = stream::iter(must_update)
        .map(|(id, slug)| {
            let client = client.clone();
            async move {
                let res = list_days(&client, &slug, first, last).await;
                (id, res)
            }
        })
        .buffer_unordered(opt.concurrent)
        .chunks(opt.menus_per_chunk);

    while let Some(chunk) = stream.next().await {
        submit_days(connection, chunk)?;
    }

    if let Some(meili_url) = &opt.meili_url {
        use database::schema::{
            days::{columns as days_cols, table as days_table},
            menus::{columns as menus_cols, table as menus_table},
        };

        let client = Client::new(meili_url, &opt.meili_key);
        let index = if let Ok(index) = client.get_index(&opt.meili_index).await {
            index
        } else {
            let task = client.create_index(&opt.meili_index, Some("id")).await?;
            let task = task
                .wait_for_completion(&client, None, Some(std::time::Duration::from_secs(10)))
                .await?;
            match task {
                Task::Enqueued { .. } | Task::Processing { .. } => {
                    bail!("timeout waiting for index creation")
                }
                Task::Failed { content } => {
                    bail!(meilisearch_sdk::errors::Error::from(content.error))
                }
                Task::Succeeded { .. } => task.try_make_index(&client).unwrap(),
            }
        };

        let mut last_days = days_table
            .group_by(days_cols::menu_id)
            .order(days_cols::menu_id.asc())
            .select((days_cols::menu_id, sql::<Date>("max(date)")))
            .load::<(MenuId, NaiveDate)>(connection)?
            .into_iter();

        index
            .set_sortable_attributes(&["updated_at", "last_day"])
            .await?;
        index
            .set_filterable_attributes(&["slug", "updated_at", "last_day"])
            .await?;

        let menus = menus_table
            .order(menus_cols::id.asc())
            .load::<Menu>(connection)?
            .into_iter();

        let menus = menus
            .map(|menu| {
                let last_day = last_days
                    .take_while_ref(|(id, _)| *id <= menu.id)
                    .find(|(id, _)| *id == menu.id)
                    .map(|(_, d)| d);

                let Menu {
                    id,
                    title,
                    slug,
                    updated_at,
                } = menu;

                MeiliMenu {
                    id,
                    updated_at,
                    slug,
                    title,
                    last_day,
                }
            })
            .collect::<Vec<_>>();
        let task = index.add_documents(&menus, None).await?;

        info!("queued {} documents for MeiliSearch indexing", menus.len());

        match task.wait_for_completion(&client, None, None).await? {
            Task::Succeeded { content } => {
                info!(
                    "indexed {} documents in {:.02} seconds",
                    menus.len(),
                    content.duration.as_secs_f64(),
                );

                Ok(())
            }
            Task::Failed { content } => bail!(meilisearch_sdk::errors::Error::from(content.error)),
            Task::Enqueued { .. } | Task::Processing { .. } => {
                bail!("timeout waiting for documents to be indexed")
            }
        }
    } else {
        Ok(())
    }
}

/// Indexes the menus from the suppliers, and stores them in the database. If
/// the menu already exists, it won't be updated. Returns the number of menus
/// inserted.
#[instrument(err, skip(connection))]
pub async fn load_menus(connection: &PgConnection) -> anyhow::Result<usize> {
    use database::schema::menus::dsl::*;

    let records = list_menus(4)
        .await?
        .into_iter()
        .map(Into::into)
        .collect::<Vec<NewMenu>>();

    info!("Fetched {} menus", records.len());

    let inserted_count = diesel::insert_into(menus)
        .values(&records)
        .on_conflict_do_nothing()
        .execute(connection)?;

    match inserted_count {
        0 => info!("no new menus were inserted"),
        _ => info!("inserted {} new menus", inserted_count),
    }

    Ok(inserted_count)
}

#[instrument(err, skip(connection))]
pub fn get_candidates(
    connection: &PgConnection,
    max_age: Duration,
    limit: Option<i64>,
) -> QueryResult<Vec<(MenuId, hugin::MenuSlug)>> {
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

#[instrument(err, skip_all)]
pub fn submit_days(
    connection: &PgConnection,
    results: Vec<(MenuId, Result<Vec<hugin::Day>, hugin::Error>)>,
) -> QueryResult<()> {
    use database::schema::days::{columns as days_columns, table as days_table};
    use database::schema::menus::{columns as menus_columns, table as menus_table};

    let successful = results
        .iter()
        .filter_map(|(m, r)| r.as_ref().ok().map(|_| *m))
        .collect::<Vec<_>>();

    let failed = results.len() - successful.len();

    if failed > 0 {
        warn!("{}/{} downloads failed", failed, results.len());
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

    info!("inserting {} days", records.len());

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
