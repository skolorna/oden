use clap::Parser;
use clap::Subcommand;
use diesel::prelude::*;
use diesel::PgConnection;
use dotenv::dotenv;
use export::export;
use index::index;
use sentry::types::Dsn;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

mod export;
mod index;

#[derive(Debug, Parser)]
struct Opt {
    #[structopt(long, env)]
    postgres_url: String,

    #[structopt(long, env)]
    sentry_dsn: Option<Dsn>,

    #[structopt(long, env)]
    sentry_environment: Option<String>,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Fetch new menus and load new days
    Index(index::Args),
    /// Export menus and days from the database
    Export(export::Args),
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

    let connection = PgConnection::establish(&opt.postgres_url)?;

    database::run_migrations(&connection).expect("migrations failed");

    match opt.cmd {
        Command::Index(opt) => index(&connection, &opt).await?,
        Command::Export(opt) => export(&connection, &opt)?,
    }

    Ok(())
}
