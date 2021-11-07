use std::sync::Arc;

use atomic_counter::AtomicCounter;
use atomic_counter::RelaxedCounter;
use butler_indexer::finalize;
use butler_indexer::get_candidates;
use butler_indexer::load_menus;
use butler_indexer::IndexerResult;
use butler_lib::errors::ButlerResult;
use butler_lib::menus::list_days;
use butler_lib::types::day::Day;
use chrono::Duration;
use chrono::TimeZone;
use chrono::Utc;
use chrono_tz::Europe::Stockholm;

use database::models::menu::Menu;
use database::models::menu::MenuId;
use database::MeiliIndexable;
use diesel::prelude::*;
use diesel::PgConnection;
use dotenv::dotenv;
use futures::stream;
use futures::StreamExt;
use meilisearch_sdk::client::Client;
use structopt::StructOpt;

use tracing::info;
use tracing::log::error;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, env)]
    postgres_url: String,

    /// If provided, the menus will be inserted into the given MeiliSearch instance.
    #[structopt(long, env)]
    meili_url: Option<String>,

    #[structopt(long, env, hide_env_values = true, default_value = "")]
    meili_key: String,

    /// Download new menus and insert them, if not already present
    #[structopt(long)]
    load_menus: bool,

    /// How many days to fetch for each menu
    #[structopt(long, default_value = "90")]
    days: u32,

    #[structopt(long, default_value = "100")]
    concurrent: usize,

    #[structopt(long, short)]
    limit: Option<i64>,
}

#[tokio::main]
async fn main() -> IndexerResult<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let opt = Opt::from_args();
    let connection = PgConnection::establish(&opt.postgres_url).unwrap();

    if opt.load_menus {
        load_menus(&connection).await?;
    }

    let must_update = get_candidates(&connection, opt.limit)?;

    info!("Updating {} menus ...", must_update.len());

    let utc = Utc::now().naive_utc().date();
    let first = Stockholm.from_utc_date(&utc).naive_local();
    let last = first + Duration::days(opt.days as i64);

    let completed_count = Arc::new(RelaxedCounter::new(0));
    let total_count = must_update.len();

    let results = stream::iter(must_update)
        .map(|(id, slug)| {
            let completed_count = completed_count.clone();
            async move {
                let d = list_days(&slug, first, last).await;
                completed_count.inc();
                info!("{}/{}", completed_count.get(), total_count);
                (id, d)
            }
        })
        .buffer_unordered(opt.concurrent)
        .collect::<Vec<(MenuId, ButlerResult<Vec<Day>>)>>()
        .await;

    finalize(&connection, results)?;

    if let Some(meili_url) = opt.meili_url {
        use database::schema::menus::dsl::*;

        let client = Client::new(&meili_url, &opt.meili_key);
        let index = client.get_or_create(Menu::MEILI_INDEX).await?;

        let documents: Vec<Menu> = menus.load(&connection)?;

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
            }
            _ => error!("Failed to index documents"),
        }
    }

    Ok(())
}
