pub mod errors;
pub mod menus;
pub mod routes;
pub mod util;

#[macro_export]
macro_rules! create_app {
    () => {{
        use actix_cors::Cors;
        use actix_web::dev::Service;
        use actix_web::http::header::{self, HeaderValue};
        use actix_web::middleware::{self, normalize};
        use actix_web::App;
        use const_format::concatcp;
        use futures::future::FutureExt;

        use menu_proxy::routes;

        const SERVER_NAME: &str = concatcp!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

        App::new()
            .wrap(normalize::NormalizePath::new(
                normalize::TrailingSlash::Trim,
            ))
            .configure(routes::configure)
            .wrap(
                Cors::default()
                    .send_wildcard()
                    .allow_any_origin()
                    .allow_any_method(),
            )
            .wrap_fn(|req, srv| {
                srv.call(req).map(|res| {
                    res.map(|mut res| {
                        let headers = res.headers_mut();
                        headers.insert(header::SERVER, HeaderValue::from_str(SERVER_NAME).unwrap());
                        res
                    })
                })
            })
            .wrap(middleware::Logger::new("%r (%Ts)"))
            .wrap(middleware::Compress::default())
    }};
}
