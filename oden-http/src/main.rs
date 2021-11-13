use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use actix_web::HttpServer;
use oden_http::create_app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8000));
    eprintln!("Binding {}", addr);

    HttpServer::new(move || create_app!())
        .bind(addr)?
        .run()
        .await
}
