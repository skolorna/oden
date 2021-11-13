use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use actix_web::HttpServer;
use database::run_migrations;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use dotenv::dotenv;
use oden_http::create_app;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, env, hide_env_values = true)]
    postgres_url: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let opt = Opt::from_args();

    let manager = ConnectionManager::<PgConnection>::new(&opt.postgres_url);
    let pool = Pool::new(manager).expect("failed to build pool");

    run_migrations(&pool.get().unwrap()).expect("migrations failed");

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8000));
    eprintln!("Binding {}", addr);

    HttpServer::new(move || create_app!(pool))
        .bind(addr)?
        .run()
        .await
}
