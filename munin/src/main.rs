use clap::{Parser, Subcommand};
use munin::index::{self, index};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser, Debug)]
struct Cli {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    Index(index::Args),
}

impl Action {
    pub async fn run(self) -> anyhow::Result<()> {
        match self {
            Self::Index(args) => index(args).await,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv();

    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .with(EnvFilter::from_default_env())
        .init();

    let args = Cli::parse();

    args.action.run().await
}
