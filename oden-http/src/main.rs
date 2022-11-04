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

#[derive(Debug, Clone, Parser)]
struct Opt {
    #[arg(long, env, hide_env_values = true)]
    postgres_url: String,

    #[arg(long, env)]
    meili_url: String,

    #[arg(long, env, hide_env_values = true, default_value = "")]
    meili_key: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let opt = Opt::parse();

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
