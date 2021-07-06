use actix_web::HttpServer;
use menu_proxy::create_app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("helo");

    HttpServer::new(move || create_app!())
        .bind("0.0.0.0:8000")?
        .run()
        .await
}
