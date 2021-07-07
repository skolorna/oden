use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use actix_web::HttpServer;
use menu_proxy::create_app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ascii_art = r#" __  __                    ____                      
|  \/  | ___ _ __  _   _  |  _ \ _ __ _____  ___   _ 
| |\/| |/ _ \ '_ \| | | | | |_) | '__/ _ \ \/ / | | |
| |  | |  __/ | | | |_| | |  __/| | | (_) >  <| |_| |
|_|  |_|\___|_| |_|\__,_| |_|   |_|  \___/_/\_\\__, |
                                               |___/ "#;
    eprintln!("{}", ascii_art);

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8000));
    eprintln!("Binding {}", addr);

    HttpServer::new(move || create_app!())
        .bind(addr)?
        .run()
        .await
}
