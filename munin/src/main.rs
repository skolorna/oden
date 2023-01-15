use clap::Parser;
use clap::Subcommand;
use dotenv::dotenv;
use munin::import::import;
use munin::{import, index};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use index::index;

#[derive(Debug, Parser)]
struct Opt {
    #[arg(long, env)]
    database_url: String,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Import(import::Args),

    /// Fetch new menus and days
    Index(index::Args),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let opt = Opt::parse();

    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env());

    tracing_subscriber::registry().with(fmt_layer).init();

    let pool = PgPoolOptions::new().connect(&opt.database_url).await?;

    stor::db::MIGRATOR.run(&pool).await?;

    match opt.cmd {
        Command::Import(opt) => {
            import(opt, &pool).await?;
        }
        Command::Index(opt) => {
            index(opt, &pool).await?;
        }
    }

    pool.close().await;

    Ok(())
}
