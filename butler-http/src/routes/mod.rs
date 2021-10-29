//! All Actix Web routes.

pub mod menus;

use actix_web::{
    http::header::{CacheControl, CacheDirective, CONTENT_TYPE},
    web, HttpResponse, Responder,
};

pub async fn get_health() -> impl Responder {
    HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::NoStore]))
        .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
        .body("\u{41f}\u{43e}\u{435}\u{445}\u{430}\u{43b}\u{438}!") // "Поехали!", russian for "Let's go!"
}

/// Configure all the routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/health").route(web::get().to(get_health)))
        .service(web::scope("/menus").configure(menus::configure));
}

/// `stale-while-revalidate` as a [CacheDirective].
pub fn swr(seconds: u32) -> CacheDirective {
    CacheDirective::Extension(
        "stale-while-revalidate".to_owned(),
        Some(seconds.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn health_ok() {
        use actix_web::{http::StatusCode, test, web, App};

        let mut app =
            test::init_service(App::new().service(web::resource("/health").to(get_health))).await;

        let req = test::TestRequest::with_uri("/health").to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("cache-control")
                .map(|h| h.to_str().unwrap()),
            Some("no-store")
        );
    }

    #[test]
    fn swr_works() {
        assert_eq!("stale-while-revalidate=300", swr(300).to_string());
        assert_eq!("stale-while-revalidate=86400", swr(86_400).to_string());
    }
}
