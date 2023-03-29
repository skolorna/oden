use std::{str::FromStr, sync::Arc};

use anyhow::Context;
use futures::{Stream, StreamExt, TryStreamExt};
use geo::VincentyDistance;
use milli::{heed::RoTxn, FieldsIdsMap, TermsMatchingStrategy};
use opentelemetry::propagation::Injector;
use reqwest::Client;
use sqlx::{Acquire, PgConnection, PgExecutor, PgPool};
use stor::{Day, Menu};
use time::{Date, Duration, OffsetDateTime};
use time_tz::OffsetDateTimeExt;
use tonic::{
    codegen::StdError,
    metadata::{MetadataKey, MetadataMap},
    transport::Channel,
    IntoRequest,
};
use tracing::{error, info, instrument, warn, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use trast_proto::{trast_client::TrastClient, NerInput};

use crate::{
    geosearch::{self, Hit},
    supplier::ListDays,
    Result,
};

mod meili;

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

    #[arg(long, default_value = "3600")]
    backoff_secs: i64,

    /// If provided, the menus will be inserted into the given
    /// MeiliSearch instance.
    #[arg(long, env)]
    meili_url: Option<String>,

    #[arg(long, env, hide_env_values = true, default_value = "")]
    meili_key: String,

    #[arg(long, env)]
    trast_url: Option<String>,
}

pub const INSERTION_BATCH_SIZE: usize = 10_000;

struct SearchTxn<'a> {
    index: &'a milli::Index,
    rtxn: RoTxn<'a>,
    fields_ids_map: FieldsIdsMap,
    trast_client: Option<TrastClient<Channel>>,
}

impl<'a> SearchTxn<'a> {
    pub async fn new<D>(index: &'a geosearch::Index, trast_url: Option<D>) -> Result<SearchTxn<'a>>
    where
        D: TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let index = &index.inner;
        let rtxn = index.read_txn()?;
        let fields_ids_map = index.fields_ids_map(&rtxn)?;
        let trast_client = if let Some(u) = trast_url {
            Some(TrastClient::connect(u).await?)
        } else {
            None
        };

        Ok(Self {
            index,
            rtxn,
            fields_ids_map,
            trast_client,
        })
    }
}

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
            created_at: _,
            checked_at: _,
            consecutive_failures: _,
        } = menu;

        assert!(osm_id.is_none());

        let (longitude, latitude) = match location {
            Some(p) => (Some(p.x()), Some(p.y())),
            None => (None, None),
        };

        sqlx::query!(
            r#"
                INSERT INTO menus (id, title, supplier, supplier_reference, longitude, latitude)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (id) DO UPDATE SET
                    title = excluded.title,
                    supplier = excluded.supplier,
                    supplier_reference = excluded.supplier_reference,
                    longitude = excluded.longitude,
                    latitude = excluded.latitude
                WHERE menus.consecutive_failures > 0 -- only update if the menu is broken
            "#,
            id,
            title,
            supplier as _,
            supplier_reference,
            longitude,
            latitude
        )
        .execute(&mut txn)
        .await
        .context("failed to insert menus")?;
    }

    Ok(txn.commit().await?)
}

fn get_expired<'a>(
    conn: impl PgExecutor<'a> + 'a,
    max_age: Duration,
    backoff: Duration,
    limit: Option<i64>,
) -> impl Stream<Item = Result<Menu>> + 'a {
    let expires_at = OffsetDateTime::now_utc() - max_age;

    sqlx::query_as::<_, Menu>(
        r#"
                SELECT * FROM menus
                WHERE
                    checked_at IS NULL OR
                    checked_at < $1 + $2 * (2 ^ (LEAST(consecutive_failures, 4)) - 1)
                LIMIT $3
            "#,
    )
    .bind(expires_at)
    .bind(backoff)
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
        Duration::seconds(opt.backoff_secs),
        opt.menu_limit,
    );

    let client = Client::new();
    let start = OffsetDateTime::now_utc().to_timezone(crate::TZ).date();
    let end = start + Duration::days(opt.days.into());

    let geoindex = geoindex.await??;
    let search_txn = match geoindex.as_ref() {
        Some(i) => Some(Arc::new(SearchTxn::new(i, opt.trast_url).await?)),
        None => None,
    };

    let mut results = expired
        .map(|result| {
            let client = client.clone();
            let search_txn = search_txn.clone();
            async move {
                match result {
                    Ok(mut menu) => {
                        let days = process_menu(
                            &client,
                            &mut menu,
                            start,
                            end,
                            opt.days,
                            search_txn.as_deref(),
                        )
                        .await;

                        Ok((menu, days))
                    }
                    Err(e) => Err(e),
                }
            }
        })
        .buffer_unordered(opt.concurrent);

    // open a new connection since get_expired uses the current
    let mut txn = pool.begin().await?;
    let mut uncommitted_queries = 0usize;

    let pb = indicatif::ProgressBar::new_spinner()
        .with_style(
            indicatif::ProgressStyle::with_template("{spinner} {msg} ({pos} done)").unwrap(),
        )
        .with_message("updating menus");

    while let Some(res) = results.next().await {
        pb.inc(1);

        let (menu, days) = match res {
            Ok(o) => o,
            Err(_) => continue,
        };

        let success = days.is_ok();
        match days {
            Ok(days) => {
                for day in days {
                    let Day { date, meals } = day;

                    sqlx::query!(
                        r#"
                            DELETE FROM meals WHERE menu_id = $1 AND date = $2
                        "#,
                        menu.id,
                        date
                    )
                    .execute(&mut txn)
                    .await
                    .context("failed to delete old meals")?;

                    for meal in meals {
                        sqlx::query!(
                            r#"
                                INSERT INTO meals (menu_id, date, meal)
                                    VALUES ($1, $2, $3)
                                    ON CONFLICT DO NOTHING
                            "#,
                            menu.id,
                            date,
                            meal
                        )
                        .execute(&mut txn)
                        .await
                        .context("failed to insert meal")?;

                        uncommitted_queries += 1;
                    }

                    if uncommitted_queries >= INSERTION_BATCH_SIZE {
                        txn.commit().await?;
                        uncommitted_queries = 0;
                        txn = pool.begin().await?;
                    }
                }
            }
            Err(e) => {
                warn!(supplier = ?menu.supplier, menu = %menu.id, supplier_reference = ?menu.supplier_reference, "{e}");
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
            created_at: _,
            checked_at: _,
            consecutive_failures: _,
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
                osm_id = $5,
                consecutive_failures = CASE
                    WHEN $6 THEN 0
                    ELSE consecutive_failures + 1
                END
            WHERE id = $7",
            now,
            title,
            longitude,
            latitude,
            osm_id,
            success,
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
            .set_sortable_attributes(&["checked_at", "last_day", "_geo"])
            .await?
            .wait_for_completion(&client, None, None)
            .await?;
        menus_index
            .set_filterable_attributes(&["slug", "checked_at", "last_day", "_geo"])
            .await?
            .wait_for_completion(&client, None, None)
            .await?;

        let menus = sqlx::query_as::<_, meili::Menu>(
            r#"
                SELECT m.*, MAX(d.date) AS last_day FROM menus AS m
                LEFT JOIN meals AS d ON d.menu_id = m.id
                GROUP BY m.id
            "#,
        )
        .fetch_all(pool)
        .await?;

        meili::add_documents(&menus_index, &menus, Some("id")).await?;
    }

    Ok(())
}

#[instrument(skip(client, menu, search_txn), fields(menu = %menu.id))]
async fn process_menu(
    client: &Client,
    menu: &mut Menu,
    start: Date,
    end: Date,
    num_days: u32,
    search_txn: Option<&SearchTxn<'_>>,
) -> Result<Vec<Day>> {
    let days = if num_days > 0 {
        let ListDays { days, menu: patch } =
            crate::list_days(client, menu.supplier, &menu.supplier_reference, start..=end).await?;

        menu.patch(patch);

        days
    } else {
        vec![]
    };

    if let Some(txn) = search_txn {
        let execute_search = |search: &milli::Search| -> Result<Option<Hit>> {
            let result = search.execute()?;
            let mut hits = txn
                .index
                .documents(&txn.rtxn, result.documents_ids)?
                .into_iter()
                .map(|(_id, obkv)| geosearch::parse_obkv(&txn.fields_ids_map, obkv));

            Ok(hits.next())
        };

        let recognized_entities = if let Some(mut trast) = txn.trast_client.clone() {
            let span = Span::current();

            let mut request = NerInput {
                sentence: menu.title.clone(),
            }
            .into_request();

            opentelemetry::global::get_text_map_propagator(|propagator| {
                propagator.inject_context(
                    &span.context(),
                    &mut MetadataInjector::new(request.metadata_mut()),
                )
            });

            let res = trast.ner(request).await?;
            Some(
                res.into_inner()
                    .entities
                    .into_iter()
                    .map(|e| e.word)
                    .collect::<Vec<String>>()
                    .join(" "),
            )
        } else {
            None
        };

        let mut search = milli::Search::new(&txn.rtxn, txn.index);
        search.limit(1);

        if let Some(hit) = recognized_entities
            .iter()
            .map(|q| (q, TermsMatchingStrategy::Any))
            .chain([
                (&menu.title, TermsMatchingStrategy::Last),
                (&menu.title, TermsMatchingStrategy::Size),
                (&menu.title, TermsMatchingStrategy::Any),
            ])
            .find_map(|(query, strategy)| {
                search.query(query);
                search.terms_matching_strategy(strategy);
                execute_search(&search).transpose()
            })
            .transpose()?
        {
            if let Some(location) = menu.location {
                if menu.osm_id.is_some()
                    || location.vincenty_distance(&hit.coordinates)? < CONVERGENCE_LIMIT_M
                {
                    menu.osm_id = Some(hit.id);
                }
            } else {
                menu.location = Some(hit.coordinates);
                menu.osm_id = Some(hit.id);
            }
        };
    }

    Ok(days)
}

struct MetadataInjector<'a> {
    metadata: &'a mut MetadataMap,
}

impl<'a> MetadataInjector<'a> {
    pub fn new(metadata: &'a mut MetadataMap) -> Self {
        Self { metadata }
    }
}

impl<'a> Injector for MetadataInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = MetadataKey::from_str(key) {
            if let Ok(value) = value.parse() {
                self.metadata.append(key, value);
            }
        }
    }
}
