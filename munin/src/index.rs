use anyhow::Context;
use futures::{Stream, StreamExt, TryStreamExt};
use geo::VincentyDistance;
use milli::TermsMatchingStrategy;
use reqwest::Client;
use sqlx::{Acquire, PgConnection, PgExecutor, PgPool};
use stor::{Day, Menu};
use time::{Duration, OffsetDateTime};
use time_tz::OffsetDateTimeExt;
use tracing::{error, info, warn};

use crate::{
    geosearch::{self, Hit},
    supplier::ListDays,
    Result,
};

const CONVERGENCE_LIMIT_M: f64 = 1000.;

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

pub const INSERTION_BATCH_SIZE: usize = 10_000;

async fn load_menus(conn: &mut PgConnection) -> anyhow::Result<()> {
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
            Some(p) => (Some(p.x()), Some(p.y())),
            None => (None, None),
        };

        let osm_id = osm_id.map(|id| id.to_string());

        sqlx::query!(
            r#"
                INSERT INTO menus (id, title, supplier, supplier_reference, longitude, latitude, osm_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (id) DO UPDATE SET
                    title = excluded.title,
                    supplier = excluded.supplier,
                    supplier_reference = excluded.supplier_reference,
                    longitude = excluded.longitude,
                    latitude = excluded.latitude,
                    osm_id = excluded.osm_id
            "#,
            id,
            title,
            supplier as _,
            supplier_reference,
            longitude,
            latitude,
            osm_id
        )
        .execute(&mut txn)
        .await.context("failed to insert menus")?;
    }

    Ok(txn.commit().await?)
}

fn get_expired<'a>(
    conn: impl PgExecutor<'a> + 'a,
    max_age: Duration,
    limit: Option<i64>,
) -> impl Stream<Item = Result<Menu>> + 'a {
    let expires_at = OffsetDateTime::now_utc() - max_age;

    sqlx::query_as::<_, Menu>(
        "SELECT * FROM menus WHERE checked_at < $1 OR checked_at IS NULL LIMIT $2",
    )
    .bind(expires_at)
    .bind(limit)
    .fetch(conn)
    .map_err(Into::into)
}

pub async fn index(opt: Args, pool: &PgPool) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;

    let gh_pat = opt.osm_gh_pat.clone();

    let geoindex = tokio::spawn(async move {
        anyhow::Ok(if let Some(ref gh_pat) = gh_pat {
            info!("building geoindex");

            match crate::geosearch::build_index(gh_pat).await {
                Ok(index) => {
                    let rtxn = index.inner.read_txn()?;
                    let num_docs = index.inner.number_of_documents(&rtxn)?;
                    info!(num_docs, "built geoindex");
                    drop(rtxn);
                    Some(index)
                }
                Err(e) => {
                    error!("failed to build geoindex: {e}");
                    None
                }
            }
        } else {
            warn!("skipping geosearch (no personal access token found)");
            None
        })
    });

    if opt.load_menus {
        load_menus(&mut conn).await?;
    }

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
                        if opt.days == 0 {
                            return Ok((menu, vec![]));
                        }

                        match crate::list_days(
                            &client,
                            menu.supplier,
                            &menu.supplier_reference,
                            start..=end,
                        )
                        .await {
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

    let geoindex = geoindex.await??;
    let search_txn = geoindex
        .as_ref()
        .map(|index| {
            let index = &index.inner;
            let rtxn = index.read_txn()?;
            let fields_ids_map = index.fields_ids_map(&rtxn)?;
            Ok::<_, milli::Error>((rtxn, index, fields_ids_map))
        })
        .transpose()?;

    let pb = indicatif::ProgressBar::new_spinner()
        .with_style(
            indicatif::ProgressStyle::with_template("{spinner} {msg} ({pos} done)").unwrap(),
        )
        .with_message("updating menus");

    while let Some(res) = results.next().await {
        pb.inc(1);

        let (mut menu, days) = match res {
            Ok(o) => o,
            Err(_) => continue,
        };

        if let Some((ref rtxn, index, ref fields_ids_map)) = search_txn {
            let execute_search = |search: &milli::Search| -> Result<Option<Hit>> {
                let result = search.execute()?;
                let mut hits = index
                    .documents(rtxn, result.documents_ids)?
                    .into_iter()
                    .map(|(_id, obkv)| geosearch::parse_obkv(fields_ids_map, obkv));

                Ok(hits.next())
            };

            let mut search = milli::Search::new(rtxn, index);
            search.query(&menu.title);
            search.limit(1);

            if let Some(hit) = [
                TermsMatchingStrategy::Last,
                TermsMatchingStrategy::Size,
                TermsMatchingStrategy::Any,
            ]
            .into_iter()
            .find_map(|strategy| {
                search.terms_matching_strategy(strategy);
                execute_search(&search).transpose()
            })
            .transpose()?
            {
                if let Some(location) = menu.location {
                    if location.vincenty_distance(&hit.coordinates)? < CONVERGENCE_LIMIT_M {
                        menu.osm_id = Some(hit.id);
                    }
                } else {
                    menu.location = Some(hit.coordinates);
                    menu.osm_id = Some(hit.id);
                }
            }
        }

        for day in days {
            let Day { date, meals } = day;

            sqlx::query!(
                r#"
                    INSERT INTO days (menu_id, date, meals)
                    VALUES ($1, $2, $3)
                    ON CONFLICT ON CONSTRAINT days_pkey DO UPDATE
                    SET meals = excluded.meals
                "#,
                menu.id,
                date,
                meals as _
            )
            .execute(&mut txn)
            .await
            .context("failed to insert days")?;

            uncommitted_queries += 1;

            if uncommitted_queries >= INSERTION_BATCH_SIZE {
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
            Some(p) => (Some(p.x()), Some(p.y())),
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

    pb.finish_and_clear();
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
    }

    Ok(())
}

mod meili {
    use std::time::Duration;

    use anyhow::bail;
    use meilisearch_sdk::{indexes::Index, tasks::Task, Client};
    use osm::OsmId;
    use serde::{Serialize, Serializer};
    use sqlx::FromRow;
    use stor::menu::Supplier;
    use time::Date;
    use tracing::info;
    use uuid::Uuid;

    #[derive(Debug, Serialize)]
    struct Geo {
        lon: f64,
        lat: f64,
    }

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
            struct Doc<'a> {
                id: Uuid,
                title: &'a str,
                #[serde(rename = "_geo", skip_serializing_if = "Option::is_none")]
                geo: Option<Geo>,
                last_day: Option<Date>,
                supplier: Supplier,
                osm_id: Option<OsmId>,
            }

            Doc {
                id: *id,
                title,
                geo: location.map(|p| Geo {
                    lon: p.x(),
                    lat: p.y(),
                }),
                last_day: *last_day,
                supplier: *supplier,
                osm_id: *osm_id,
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
            .wait_for_completion(&index.client, None, Some(Duration::from_secs(30)))
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
