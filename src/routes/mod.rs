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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, web, App};

    #[actix_rt::test]
    async fn health_ok() {
        let mut app =
            test::init_service(App::new().service(web::resource("/health").to(get_health))).await;

        let req = test::TestRequest::with_uri("/health").to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
