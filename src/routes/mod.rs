pub mod menus;

use actix_web::{web, Responder};

pub async fn get_health() -> impl Responder {
    "Поехали!" // Russian for "Let's go!"
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/health").route(web::get().to(get_health)))
        .service(web::scope("/menus").configure(menus::configure));
}
