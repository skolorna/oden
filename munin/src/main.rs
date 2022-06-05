use diesel::prelude::*;
use diesel::PgConnection;
use dotenv::dotenv;
use exporter::export;
use exporter::ExporterOpt;
use indexer::index;
use indexer::IndexerOpt;
use structopt::StructOpt;

mod exporter;
mod indexer;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, env)]
    postgres_url: String,

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
    tracing_subscriber::fmt::init();

    let opt = Opt::from_args();
    let connection = PgConnection::establish(&opt.postgres_url).unwrap();

    database::run_migrations(&connection).expect("migrations failed");

    match opt.cmd {
        Command::Index(opt) => index(&connection, &opt).await?,
        Command::Export(opt) => export(&connection, &opt)?,
    }

    Ok(())
}
