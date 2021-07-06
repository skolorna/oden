//! All Actix Web routes.

pub mod menus;

use actix_web::{web, Responder};

pub async fn get_health() -> impl Responder {
    "\u{41f}\u{43e}\u{435}\u{445}\u{430}\u{43b}\u{438}!" // "Поехали!", russian for "Let's go!"
}

/// Configure all the  routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/health").route(web::get().to(get_health)))
        .service(web::scope("/menus").configure(menus::configure));
}
