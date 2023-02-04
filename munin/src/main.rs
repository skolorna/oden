use clap::Parser;
use clap::Subcommand;
use dotenv::dotenv;
use munin::import::import;
use munin::{import, index};
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::trace;
use opentelemetry::sdk::trace::Sampler;
use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use sqlx::postgres::PgPoolOptions;
use tracing::metadata::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use index::index;

#[derive(Debug, Parser)]
struct Opt {
    #[arg(long, env)]
    database_url: String,

    #[command(subcommand)]
    cmd: Command,

    #[arg(long, env)]
    otlp_endpoint: Option<String>,
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

    if let Some(otlp_endpoint) = opt.otlp_endpoint {
        init_telemetry(otlp_endpoint)?;
    }

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

fn init_telemetry(otlp_endpoint: impl Into<String>) -> anyhow::Result<()> {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(otlp_endpoint);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(0.1))
                .with_resource(Resource::new(vec![
                    KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                        env!("CARGO_PKG_NAME"),
                    ),
                    KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
                        env!("CARGO_PKG_VERSION"),
                    ),
                ])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(fmt::layer())
        .with(otel_layer)
        .init();

    Ok(())
}
