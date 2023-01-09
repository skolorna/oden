use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use clap::Subcommand;
use dotenv::dotenv;
use munin::index;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use index::index;

#[derive(Debug, Parser)]
struct Opt {
    #[arg(long, env)]
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

    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env());

    tracing_subscriber::registry().with(fmt_layer).init();

    let path = opt.db_path.unwrap();
    let sqlite_url = format!("sqlite://{}", path.display());

    let options = SqliteConnectOptions::from_str(&sqlite_url)?.create_if_missing(opt.create_db);
    let pool = SqlitePool::connect_with(options).await?;

    stor::db::MIGRATOR.run(&pool).await?;

    match opt.cmd {
        Command::Index(opt) => {
            index(&opt, &pool).await?;
        }
    }

    pool.close().await;

    Ok(())
}
