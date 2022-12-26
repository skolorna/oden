use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use clap::Subcommand;
use dotenv::dotenv;
use munin::index;
use sentry::types::Dsn;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

mod meili;

use index::index;

#[derive(Debug, Parser)]
struct Opt {
    #[arg(long, env)]
    sentry_dsn: Option<Dsn>,

    #[arg(long, env)]
    sentry_environment: Option<String>,

    #[arg(long)]
    db_path: Option<PathBuf>,

    #[arg(long, default_value_t)]
    create_db: bool,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Fetch new menus and days
    Index(index::Args),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let opt = Opt::parse();

    let _guard = sentry::init(sentry::ClientOptions {
        // Set this a to lower value in production
        traces_sample_rate: 1.0,
        dsn: opt.sentry_dsn,
        environment: opt.sentry_environment.map(Into::into),
        ..sentry::ClientOptions::default()
    });

    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(sentry_tracing::layer())
        .init();

    let path = opt.db_path.unwrap();
    let sqlite_url = format!("sqlite://{}", path.display());

    let options = SqliteConnectOptions::from_str(&sqlite_url)?.create_if_missing(opt.create_db);
    let pool = SqlitePool::connect_with(options).await?;

    stor::db::MIGRATOR.run(&pool).await?;

    match opt.cmd {
        Command::Index(opt) => index(&opt, &pool).await,
    }
}
