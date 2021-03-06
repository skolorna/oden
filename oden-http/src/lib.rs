use actix_web::web::Data;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

pub mod errors;
pub mod routes;

pub type PgPoolData = Data<Pool<ConnectionManager<PgConnection>>>;

#[macro_export]
macro_rules! create_app {
    ($pool:expr) => {{
        use actix_web::dev::Service;
        use actix_web::http::header::{self, HeaderValue};
        use actix_web::middleware::{self, NormalizePath};
        use actix_web::web::Data;
        use actix_web::App;
        use futures::FutureExt;

        use oden_http::routes;

        App::new()
            .app_data(Data::new($pool.clone()))
            .wrap(NormalizePath::trim())
            .configure(routes::configure)
            .wrap_fn(|req, srv| {
                srv.call(req).map(|res| {
                    res.map(|mut res| {
                        let headers = res.headers_mut();
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
