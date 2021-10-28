pub mod errors;
pub mod routes;

#[macro_export]
macro_rules! create_app {
    () => {{
        use actix_web::dev::Service;
        use actix_web::http::header::{self, HeaderValue};
        use actix_web::middleware::{self, normalize};
        use actix_web::App;
        use const_format::concatcp;

        use butler_http::routes;

        const SERVER_NAME: &str = concatcp!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

        App::new()
            .wrap(normalize::NormalizePath::new(
                normalize::TrailingSlash::Trim,
            ))
            .configure(routes::configure)
            .wrap_fn(|req, srv| {
                srv.call(req).map(|res| {
                    res.map(|mut res| {
                        let headers = res.headers_mut();
                        headers.insert(header::SERVER, HeaderValue::from_str(SERVER_NAME).unwrap());
                        headers.insert(
                            header::ACCESS_CONTROL_ALLOW_ORIGIN,
                            HeaderValue::from_str("*").unwrap(),
                        );
                        res
                    })
                })
            })
            .wrap(middleware::Logger::new("%r (%Ts)"))
            .wrap(middleware::Compress::default())
    }};
}
