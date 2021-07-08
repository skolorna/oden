pub mod errors;
pub mod menus;
pub mod routes;
pub mod util;

#[macro_export]
macro_rules! create_app {
    () => {{
        use actix_cors::Cors;
        use actix_web::middleware::{self, normalize};
        use actix_web::App;
        use menu_proxy::routes;

        App::new()
            .wrap(normalize::NormalizePath::new(
                normalize::TrailingSlash::Trim,
            ))
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
