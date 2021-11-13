use munin::get_candidates;
use munin::load_menus;
use munin::submit_days;
use munin::IndexerResult;

use munin_lib::menus::list_days;

use chrono::Duration;
use chrono::TimeZone;
use chrono::Utc;
use chrono_tz::Europe::Stockholm;

use database::models::menu::Menu;
use database::MeiliIndexable;
use diesel::prelude::*;
use diesel::PgConnection;
use dotenv::dotenv;
use futures::stream;
use futures::StreamExt;

use meilisearch_sdk::client::Client;
use structopt::StructOpt;

use tracing::error;
use tracing::info;

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

    #[structopt(long, default_value = "50")]
    concurrent: usize,

    #[structopt(long, default_value = "500")]
    batch_size: usize,

    #[structopt(long, short)]
    limit: Option<i64>,

    #[structopt(long, default_value = "86400")]
    max_age_secs: i64,
}

#[tokio::main]
async fn main() -> IndexerResult<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let opt = Opt::from_args();
    let connection = PgConnection::establish(&opt.postgres_url).unwrap();

    database::run_migrations(&connection).expect("migrations failed");

    if opt.load_menus {
        load_menus(&connection).await?;
    }

    let must_update = get_candidates(&connection, Duration::seconds(opt.max_age_secs), opt.limit)?;

    info!("Updating {} menus ...", must_update.len());

    let utc = Utc::now().naive_utc().date();
    let first = Stockholm.from_utc_date(&utc).naive_local();
    let last = first + Duration::days(opt.days as i64);

    let stream = stream::iter(must_update)
        .map(|(id, slug)| async move {
            let d = list_days(&slug, first, last).await;
            (id, d)
        })
        .buffer_unordered(opt.concurrent);

    stream
        .chunks(opt.batch_size)
        .for_each(|chunk| async {
            submit_days(&connection, chunk).unwrap();
        })
        .await;

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
