use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use diesel::r2d2::ConnectionManager;
use actix_web::HttpServer;
use oden_http::create_app;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, env, hide_env_values = true)]
    postgres_url: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt::init();

    panic!();

    let opt = Opt::from_args();

    let manager = ConnectionManager::new(&opt.postgres_url);

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8000));
    eprintln!("Binding {}", addr);

    HttpServer::new(move || create_app!())
        .bind(addr)?
        .run()
        .await
}
