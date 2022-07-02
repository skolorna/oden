use diesel::prelude::*;
use diesel::PgConnection;
use dotenv::dotenv;
use exporter::export;
use exporter::ExporterOpt;
use indexer::index;
use indexer::IndexerOpt;
use sentry::types::Dsn;
use structopt::StructOpt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

mod exporter;
mod indexer;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, env)]
    postgres_url: String,

    #[structopt(env)]
    sentry_dsn: Option<Dsn>,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    Index(IndexerOpt),
    Export(ExporterOpt),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let opt = Opt::from_args();

    let _guard = sentry::init(sentry::ClientOptions {
        // Set this a to lower value in production
        traces_sample_rate: 1.0,
        dsn: opt.sentry_dsn,
        ..sentry::ClientOptions::default()
    });

    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(sentry_tracing::layer())
        .init();

    let connection = PgConnection::establish(&opt.postgres_url).unwrap();

    database::run_migrations(&connection).expect("migrations failed");

    match opt.cmd {
        Command::Index(opt) => index(&connection, &opt).await?,
        Command::Export(opt) => export(&connection, &opt)?,
    }

    Ok(())
}
