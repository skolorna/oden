//! All Actix Web routes.

pub mod menus;

use std::iter::FromIterator;

use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web, HttpResponse, Responder,
};
use diesel::{sql_query, sql_types, Queryable, QueryableByName, RunQueryDsl};
use serde::Serialize;

use crate::{
    errors::{AppError, AppResult},
    PgPoolData,
};

#[derive(Debug, Serialize)]
struct HealthResponse {
    version: &'static str,
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

pub async fn get_health() -> impl Responder {
    HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::NoStore]))
        .json(HealthResponse::default())
}

#[derive(Debug, Serialize)]
struct Stats {
    menus: i64,
    days: i64,
}

impl FromIterator<CatalogRow> for Option<Stats> {
    fn from_iter<T: IntoIterator<Item = CatalogRow>>(iter: T) -> Self {
        use database::schema;

        let mut menus = None;
        let mut days = None;

        for CatalogRow { reltuples, relname } in iter {
            match relname.as_str() {
                schema::DAYS_TABLE => days = Some(reltuples),
                schema::MENUS_TABLE => menus = Some(reltuples),
                _ => {}
            }
        }

        match (menus, days) {
            (Some(menus), Some(days)) => Some(Stats { menus, days }),
            _ => None,
        }
    }
}

#[derive(Debug, Queryable, QueryableByName)]
#[diesel(table_name = "pg_class")]
struct CatalogRow {
    #[sql_type = "sql_types::BigInt"]
    reltuples: i64,
    #[sql_type = "sql_types::Text"]
    relname: String,
}

pub async fn get_stats(pool: PgPoolData) -> AppResult<HttpResponse> {
    use database::schema;

    let conn = pool.get()?;

    let rows = web::block(move || {
        sql_query("SELECT reltuples::bigint, relname FROM pg_class where relname IN ($1, $2)")
            .bind::<sql_types::Text, _>(schema::DAYS_TABLE)
            .bind::<sql_types::Text, _>(schema::MENUS_TABLE)
            .load::<CatalogRow>(&conn)
    })
    .await??;

    let stats: Stats = match rows.into_iter().collect() {
        Some(stats) => stats,
        None => return Err(AppError::InternalError),
    };

    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::MaxAge(600)]))
        .json(stats))
}

/// Configure all the routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/health").route(web::get().to(get_health)))
        .service(web::resource("/stats").route(web::get().to(get_stats)))
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

        let app =
            test::init_service(App::new().service(web::resource("/health").to(get_health))).await;

        let req = test::TestRequest::with_uri("/health").to_request();
        let resp = test::call_service(&app, req).await;

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
