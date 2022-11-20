use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use actix_web::HttpServer;
use clap::Parser;
use database::run_migrations;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use dotenv::dotenv;
use oden_http::create_app;
use sentry::types::Dsn;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[derive(Debug, Clone, Parser)]
struct Opt {
    #[arg(long, env, hide_env_values = true)]
    postgres_url: String,

    #[arg(long, env)]
    meili_url: String,

    #[arg(long, env, hide_env_values = true, default_value = "")]
    meili_key: String,

    #[arg(env)]
    sentry_dsn: Option<Dsn>,

    #[arg(env)]
    sentry_environment: Option<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let opt = Opt::parse();

    let _guard = sentry::init(sentry::ClientOptions {
        dsn: opt.sentry_dsn.clone(),
        environment: opt.sentry_environment.clone().map(Into::into),
        traces_sample_rate: 1.0,
        release: sentry::release_name!(),
        ..Default::default()
    });

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env()))
        .with(sentry_tracing::layer())
        .init();

    let manager = ConnectionManager::<PgConnection>::new(&opt.postgres_url);
    let pool = Pool::new(manager).expect("failed to build pool");

    run_migrations(&pool.get().unwrap()).expect("migrations failed");

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8000));
    eprintln!("Binding {}", addr);

    HttpServer::new(move || create_app!(pool, &opt.meili_url, &opt.meili_key))
        .bind(addr)?
        .run()
        .await
}
