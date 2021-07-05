pub mod menus;
pub mod routes;

use actix_cors::Cors;
use actix_web::middleware;
use actix_web::{App, HttpServer};

#[macro_export]
macro_rules! create_app {
    () => {{
        App::new()
            .configure(routes::configure)
            .wrap(
                Cors::default()
                    .send_wildcard()
                    .allow_any_origin()
                    .allow_any_method()
                    .max_age(86_400), // 24h
            )
            .wrap(middleware::Compress::default())
    }};
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("helo");

    HttpServer::new(move || create_app!())
        .bind("0.0.0.0:8000")?
        .run()
        .await
}
