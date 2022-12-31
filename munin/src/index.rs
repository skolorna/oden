use futures::{Stream, StreamExt, TryStreamExt};
use reqwest::Client;
use sqlx::{Acquire, SqliteConnection, SqliteExecutor, SqlitePool};
use stor::{menu::Coord, Day, Menu};
use time::{Duration, OffsetDateTime};
use time_tz::OffsetDateTimeExt;
use tracing::{error, info, warn};

use crate::{geosearch, supplier::ListDays, Result};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Download new menus and insert them, if not already present.
    #[arg(long)]
    load_menus: bool,

    /// GitHub personal access token for the OSM repository, enabling
    /// geosearch for menus.
    #[arg(env)]
    osm_gh_pat: Option<String>,

    /// How many days to fetch for each menu
    #[arg(long, default_value = "90")]
    days: u32,

    #[arg(long, default_value = "50")]
    concurrent: usize,

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
}

const DAYS_BATCH_INSERT_COUNT: usize = 1_000;

async fn load_menus(conn: &mut SqliteConnection) -> anyhow::Result<()> {
    let menus = crate::list_menus(4).await?;

    let mut txn = conn.begin().await?;

    for menu in menus {
        let Menu {
            id,
            title,
            supplier,
            supplier_reference,
            location,
            osm_id,
        } = menu;

        let (longitude, latitude) = match location {
            Some(Coord {
                longitude,
                latitude,
            }) => (Some(longitude), Some(latitude)),
            None => (None, None),
        };

        let osm_id = osm_id.map(|id| id.to_string());

        sqlx::query!("INSERT INTO menus (id, title, supplier, supplier_reference, longitude, latitude, osm_id) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        id, title, supplier, supplier_reference, longitude, latitude, osm_id).execute(&mut txn).await?;
    }

    Ok(txn.commit().await?)
}

fn get_expired<'a>(
    conn: impl SqliteExecutor<'a> + 'a,
    max_age: Duration,
    limit: Option<i64>,
) -> impl Stream<Item = Result<Menu>> + 'a {
    let expires_at = OffsetDateTime::now_utc() - max_age;

    sqlx::query_as::<_, Menu>(
        "SELECT * FROM menus WHERE checked_at < $1 OR checked_at IS NULL LIMIT $2",
    )
    .bind(expires_at)
    .bind(limit.unwrap_or(-1))
    .fetch(conn)
    .map_err(Into::into)
}

pub async fn index(opt: &Args, pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query("PRAGMA journal_mode=WAL").execute(pool).await?;
    sqlx::query("PRAGMA busy_timeout=60000")
        .execute(pool)
        .await?;

    let mut conn = pool.acquire().await?;

    if opt.load_menus {
        load_menus(&mut conn).await?;
    }

    let geoindex = if let Some(ref gh_pat) = opt.osm_gh_pat {
        info!("building geoindex");

        match crate::geosearch::build_index(gh_pat).await {
            Ok(index) => {
                let rtxn = index.inner.read_txn()?;
                let num_docs = index.inner.number_of_documents(&rtxn)?;
                info!("built geoindex ({num_docs} documents)");
                drop(rtxn);
                Some(index)
            }
            Err(e) => {
                error!("failed to build geoindex: {e}");
                None
            }
        }
    } else {
        info!("skipping geosearch (no personal access token found)");
        None
    };

    let expired = get_expired(
        &mut conn,
        Duration::seconds(opt.max_age_secs),
        opt.menu_limit,
    );

    let client = Client::new();
    let start = OffsetDateTime::now_utc().to_timezone(crate::TZ).date();
    let end = start + Duration::days(opt.days.into());

    let mut results = expired
        .map(|result| {
            let client = client.clone();
            async move {
                match result {
                    Ok(menu) => {
                        match crate::list_days(
                            &client,
                            menu.supplier,
                            &menu.supplier_reference,
                            start..=end,
                        )
                        .await
                        {
                            Ok(ListDays {
                                days,
                                menu: patched_menu,
                            }) => Ok((patched_menu.unwrap_or(menu), days)),
                            Err(e) => {
                                warn!(supplier = ?menu.supplier, menu = %menu.id, supplier_reference = ?menu.supplier_reference, "{e}");
                                Err(e)
                            }
                        }
                    }
                    Err(e) => Err(e),
                }
            }
        })
        .buffer_unordered(opt.concurrent);

    // open a new connection since get_expired uses the current
    let mut txn = pool.begin().await?;
    let mut uncommitted_queries = 0usize;

    let search_txn = geoindex
        .as_ref()
        .map(|index| {
            let index = &index.inner;
            let rtxn = index.read_txn()?;
            let fields_ids_map = index.fields_ids_map(&rtxn)?;
            Ok::<_, milli::Error>((rtxn, index, fields_ids_map))
        })
        .transpose()?;

    while let Some(res) = results.next().await {
        let (mut menu, days) = match res {
            Ok(o) => o,
            Err(_) => continue,
        };

        if let Some((ref rtxn, index, ref fields_ids_map)) = search_txn {
            if menu.location.is_none() {
                let mut search = milli::Search::new(rtxn, index);
                search.query(&menu.title);
                search.limit(1);

                let result = search.execute()?;
                let mut hits = index
                    .documents(rtxn, result.documents_ids)?
                    .into_iter()
                    .map(|(_id, obkv)| geosearch::parse_obkv(fields_ids_map, obkv));

                if let Some(hit) = hits.next() {
                    menu.location = Some(hit.coordinates);
                    menu.osm_id = Some(hit.id);
                }
            }
        }

        for day in days {
            let Day { date, meals } = day;

            sqlx::query!(
                "INSERT OR REPLACE INTO days (menu_id, date, meals) VALUES ($1, $2, $3)",
                menu.id,
                date,
                meals
            )
            .execute(&mut txn)
            .await?;

            uncommitted_queries += 1;

            if uncommitted_queries >= DAYS_BATCH_INSERT_COUNT {
                txn.commit().await?;
                uncommitted_queries = 0;
                txn = pool.begin().await?;
            }
        }

        let now = OffsetDateTime::now_utc();

        let Menu {
            id,
            title,
            supplier: _,
            supplier_reference: _,
            location,
            osm_id,
        } = menu;

        let (longitude, latitude) = match location {
            Some(Coord {
                longitude,
                latitude,
            }) => (Some(longitude), Some(latitude)),
            None => (None, None),
        };
        let osm_id = osm_id.map(|id| id.to_string());

        sqlx::query!(
            "UPDATE menus SET
                checked_at = $1,
                title = $2,
                longitude = $3,
                latitude = $4,
                osm_id = $5
            WHERE id = $6",
            now,
            title,
            longitude,
            latitude,
            osm_id,
            id,
        )
        .execute(&mut txn)
        .await?;

        uncommitted_queries += 1;
    }

    txn.commit().await?;

    if let Some(ref meili_url) = opt.meili_url {
        let client = meilisearch_sdk::Client::new(meili_url, &opt.meili_key);

        let menus_index = meili::get_or_create_index(&client, "menus").await?;
        menus_index
            .set_sortable_attributes(&["updated_at", "last_day"])
            .await?;
        menus_index
            .set_filterable_attributes(&["slug", "updated_at", "last_day"])
            .await?;

        let menus = sqlx::query_as::<_, meili::Menu>(
            r#"
            SELECT m.*, MAX(d.date) AS last_day FROM menus AS m
            LEFT JOIN days AS d ON d.menu_id = m.id
            GROUP BY m.id
        "#,
        )
        .fetch_all(pool)
        .await?;

        meili::add_documents(&menus_index, &menus, None).await?;

        let days_index = meili::get_or_create_index(&client, "days").await?;

        let mut days = sqlx::query_as::<_, meili::Day>("SELECT menu_id, date, meals FROM days")
            .fetch(pool)
            .try_chunks(10_000);

        while let Some(chunk) = days.try_next().await? {
            meili::add_documents(&days_index, &chunk, Some("id")).await?;
        }
    }

    todo!()
}

mod meili {
    use std::time::Duration;

    use anyhow::bail;
    use meilisearch_sdk::{indexes::Index, tasks::Task, Client};
    use osm::OsmId;
    use serde::{Serialize, Serializer};
    use sqlx::FromRow;
    use stor::{
        day::Meals,
        menu::{Coord, Supplier},
    };
    use time::Date;
    use tracing::info;
    use uuid::Uuid;

    #[derive(Debug, FromRow)]
    pub struct Menu {
        #[sqlx(flatten)]
        inner: stor::Menu,
        last_day: Option<Date>,
    }

    impl Serialize for Menu {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let Self { inner, last_day } = self;
            let stor::Menu {
                id,
                title,
                supplier,
                supplier_reference: _,
                location,
                osm_id,
            } = inner;

            #[derive(Debug, Serialize)]
            struct Geo {
                lon: f64,
                lat: f64,
            }

            #[derive(Debug, Serialize)]
            struct Doc<'a> {
                id: Uuid,
                title: &'a str,
                #[serde(rename = "_geo")]
                geo: Option<Geo>,
                last_day: Option<Date>,
                supplier: Supplier,
                osm_id: Option<OsmId>,
            }

            Doc {
                id: *id,
                title,
                geo: location.map(
                    |Coord {
                         longitude,
                         latitude,
                     }| Geo {
                        lon: longitude,
                        lat: latitude,
                    },
                ),
                last_day: *last_day,
                supplier: *supplier,
                osm_id: *osm_id,
            }
            .serialize(serializer)
        }
    }

    #[derive(Debug, FromRow)]
    pub struct Day {
        menu_id: Uuid,
        date: Date,
        meals: Meals,
    }

    impl Serialize for Day {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            #[derive(Debug, Serialize)]
            struct Doc<'a> {
                id: Uuid,
                menu_id: Uuid,
                date: Date,
                meals: &'a Meals,
            }

            Doc {
                id: Uuid::new_v5(&self.menu_id, self.date.to_string().as_bytes()),
                menu_id: self.menu_id,
                date: self.date,
                meals: &self.meals,
            }
            .serialize(serializer)
        }
    }

    pub async fn add_documents<T>(
        index: &Index,
        documents: &[T],
        primary_key: Option<&str>,
    ) -> anyhow::Result<()>
    where
        T: Serialize,
    {
        let task = index.add_documents(documents, primary_key).await?;

        info!(
            "queued {} documents for meilisearch indexing",
            documents.len()
        );

        match task
            .wait_for_completion(&index.client, None, Some(Duration::from_secs(10)))
            .await?
        {
            Task::Succeeded { content } => {
                info!(
                    "indexed {} documents in {:.02} seconds",
                    documents.len(),
                    content.duration.as_secs_f64(),
                );

                Ok(())
            }
            Task::Failed { content } => bail!(meilisearch_sdk::errors::Error::from(content.error)),
            Task::Enqueued { .. } | Task::Processing { .. } => {
                bail!("timeout waiting for documents to be indexed")
            }
        }
    }

    pub async fn get_or_create_index(
        client: &Client,
        uid: impl AsRef<str>,
    ) -> anyhow::Result<Index> {
        let uid = uid.as_ref();

        if let Ok(index) = client.get_index(uid).await {
            Ok(index)
        } else {
            let task = client.create_index(uid, None).await?;
            let task = task
                .wait_for_completion(client, None, Some(std::time::Duration::from_secs(10)))
                .await?;
            match task {
                Task::Enqueued { .. } | Task::Processing { .. } => {
                    bail!("timeout waiting for index creation")
                }
                Task::Failed { content } => {
                    bail!(meilisearch_sdk::errors::Error::from(content.error))
                }
                Task::Succeeded { .. } => Ok(task.try_make_index(client).unwrap()),
            }
        }
    }
}

// #[derive(Debug, Serialize)]
// struct MeiliMenu {
//     id: MenuId,
//     updated_at: Option<DateTime<Utc>>,
//     slug: MenuSlug,
//     title: String,
//     last_day: Option<NaiveDate>,
// }

// pub async fn index(connection: &PgConnection, opt: &Args) -> anyhow::Result<()> {
//     let client = reqwest::Client::new();

//     if opt.load_menus {
//         load_menus(connection).await?;
//     }

//     let must_update = get_candidates(
//         connection,
//         Duration::seconds(opt.max_age_secs),
//         opt.menu_limit,
//     )?;

//     info!("updating {} menus", must_update.len());

//     let utc = Utc::now().naive_utc().date();
//     let first = Stockholm.from_utc_date(&utc).naive_local();
//     let last = first + Duration::days(opt.days as i64);

//     let mut stream = stream::iter(must_update)
//         .map(|(id, slug)| {
//             let client = client.clone();
//             async move {
//                 let res = list_days(&client, &slug, first, last).await;
//                 (id, res)
//             }
//         })
//         .buffer_unordered(opt.concurrent)
//         .chunks(opt.menus_per_chunk);

//     while let Some(chunk) = stream.next().await {
//         submit_days(connection, chunk)?;
//     }

//     if let Some(meili_url) = &opt.meili_url {
//         use database::schema::{
//             days::{columns as days_cols, table as days_table},
//             menus::{columns as menus_cols, table as menus_table},
//         };

//         let client = Client::new(meili_url, &opt.meili_key);
//         let index = meili::get_or_create_index(&client, &opt.meili_index).await?;

//         let mut last_days = days_table
//             .group_by(days_cols::menu_id)
//             .order(days_cols::menu_id.asc())
//             .select((days_cols::menu_id, sql::<Date>("max(date)")))
//             .load::<(MenuId, NaiveDate)>(connection)?
//             .into_iter();

//         index
//             .set_sortable_attributes(&["updated_at", "last_day"])
//             .await?;
//         index
//             .set_filterable_attributes(&["slug", "updated_at", "last_day"])
//             .await?;

//         let menus = menus_table
//             .order(menus_cols::id.asc())
//             .load::<Menu>(connection)?
//             .into_iter();

//         let menus = menus
//             .map(|menu| {
//                 let last_day = last_days
//                     .take_while_ref(|(id, _)| *id <= menu.id)
//                     .find(|(id, _)| *id == menu.id)
//                     .map(|(_, d)| d);

//                 let Menu {
//                     id,
//                     title,
//                     slug,
//                     updated_at,
//                 } = menu;

//                 MeiliMenu {
//                     id,
//                     updated_at,
//                     slug,
//                     title,
//                     last_day,
//                 }
//             })
//             .collect::<Vec<_>>();

//         meili::add_documents(&index, &menus, None).await?;

//         drop(menus);

//         let index = meili::get_or_create_index(&client, "days").await?;

//         days_chunks(connection, 10_000, |chunk| {
//             block_on(async {
//                 #[derive(Debug, Serialize)]
//                 struct Day {
//                     id: String,
//                     menu_id: MenuId,
//                     date: NaiveDate,
//                     meals: Vec<String>,
//                 }

//                 let days = chunk
//                     .into_iter()
//                     .map(|d| Day {
//                         id: format!("{}-{}", d.menu_id, d.date),
//                         menu_id: d.menu_id,
//                         date: d.date,
//                         meals: d.meals.to_string().lines().map(ToOwned::to_owned).collect(),
//                     })
//                     .collect::<Vec<_>>();

//                 meili::add_documents(&index, &days, None).await.unwrap();
//             });
//         })?;

//         Ok(())
//     } else {
//         Ok(())
//     }
// }

// /// Indexes the menus from the suppliers, and stores them in the database. If
// /// the menu already exists, it won't be updated. Returns the number of menus
// /// inserted.
// #[instrument(err, skip(connection))]
// pub async fn load_menus(connection: &PgConnection) -> anyhow::Result<usize> {
//     use database::schema::menus::dsl::*;

//     let records = list_menus(4)
//         .await?
//         .into_iter()
//         .map(Into::into)
//         .collect::<Vec<NewMenu>>();

//     info!("Fetched {} menus", records.len());

//     let inserted_count = diesel::insert_into(menus)
//         .values(&records)
//         .on_conflict_do_nothing()
//         .execute(connection)?;

//     match inserted_count {
//         0 => info!("no new menus were inserted"),
//         _ => info!("inserted {} new menus", inserted_count),
//     }

//     Ok(inserted_count)
// }

// #[instrument(err, skip(connection))]
// pub fn get_candidates(
//     connection: &PgConnection,
//     max_age: Duration,
//     limit: Option<i64>,
// ) -> QueryResult<Vec<(MenuId, hugin::MenuSlug)>> {
//     use database::schema::menus::dsl::*;

//     let q = menus
//         .select((id, slug))
//         .filter(updated_at.lt(Utc::now() - max_age).or(updated_at.is_null()));

//     if let Some(limit) = limit {
//         q.limit(limit).load(connection)
//     } else {
//         q.load(connection)
//     }
// }

// #[instrument(err, skip_all)]
// pub fn submit_days(
//     connection: &PgConnection,
//     results: Vec<(MenuId, Result<Vec<hugin::Day>, hugin::Error>)>,
// ) -> QueryResult<()> {
//     use database::schema::days::{columns as days_columns, table as days_table};
//     use database::schema::menus::{columns as menus_columns, table as menus_table};

//     let successful = results
//         .iter()
//         .filter_map(|(m, r)| r.as_ref().ok().map(|_| *m))
//         .collect::<Vec<_>>();

//     let failed = results.len() - successful.len();

//     if failed > 0 {
//         warn!("{}/{} downloads failed", failed, results.len());
//     }

//     let records = results
//         .into_iter()
//         .filter_map(|(m, r)| {
//             let d = r
//                 .ok()?
//                 .into_iter()
//                 .map(|d| NewDay::from_day(d, m))
//                 .collect::<Vec<_>>();

//             Some(d)
//         })
//         .flatten()
//         .collect::<Vec<NewDay>>();

//     info!("inserting {} days", records.len());

//     for chunk in records.chunks(10000) {
//         diesel::insert_into(days_table)
//             .values(chunk)
//             // .on_conflict_do_nothing()
//             .on_conflict((days_columns::menu_id, days_columns::date))
//             .do_update()
//             .set(days_columns::meals.eq(excluded(days_columns::meals)))
//             .execute(connection)?;
//     }

//     let updated_at = Utc::now();

//     for chunk in successful.chunks(1000) {
//         diesel::update(menus_table.filter(menus_columns::id.eq_any(chunk)))
//             .set(menus_columns::updated_at.eq(updated_at))
//             .execute(connection)?;
//     }

//     Ok(())
// }
