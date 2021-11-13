use actix_web::{
    get,
    http::header::{CacheControl, CacheDirective},
    web::{self, ServiceConfig},
    HttpResponse,
};
use chrono::{Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Europe::Stockholm;
use database::models::{self, menu::MenuId};
use diesel::prelude::*;
use munin_lib::types;
use serde::Deserialize;

use crate::{
    errors::{AppError, AppResult},
    routes::swr,
    PgPoolData,
};

/// Route for listing menus.
async fn list_menus_route(pool: PgPoolData) -> AppResult<HttpResponse> {
    use database::schema::menus::dsl::*;

    let connection = pool.get()?;
    let rows = web::block(move || menus.load::<models::menu::Menu>(&connection)).await??;

    let res = HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::MaxAge(86_400), // 1 day
            swr(604_800),                   // 7 days
        ]))
        .json(rows);

    Ok(res)
}

#[get("{menu}")]
async fn query_menu_route(menu_id: web::Path<MenuId>, pool: PgPoolData) -> AppResult<HttpResponse> {
    use database::schema::menus::dsl::*;

    let connection = pool.get()?;
    let row: Option<models::menu::Menu> = web::block(move || {
        menus
            .find(menu_id.into_inner())
            .first(&connection)
            .optional()
    })
    .await??;

    if let Some(menu) = row {
        let res = HttpResponse::Ok()
            .insert_header(CacheControl(vec![
                CacheDirective::MaxAge(86_400), // 1 day
                swr(604_800),                   // 7 days
            ]))
            .json(menu);

        Ok(res)
    } else {
        Err(AppError::MenuNotFound)
    }
}

#[derive(Deserialize)]
struct ListDaysRouteQuery {
    first: Option<NaiveDate>,
    last: Option<NaiveDate>,
}

/// Route for listing days.
#[get("{menu}/days")]
async fn list_days_route(
    menu: web::Path<MenuId>,
    query: web::Query<ListDaysRouteQuery>,
    pool: PgPoolData,
) -> AppResult<HttpResponse> {
    use database::schema::days::dsl::*;

    let first = query.first.unwrap_or_else(|| {
        let naive_now = Utc::now().naive_utc();

        Stockholm.from_utc_datetime(&naive_now).date().naive_local()
    });
    let last = query.last.unwrap_or_else(|| first + Duration::weeks(2));

    if first > last {
        return Err(AppError::BadRequest(format!(
            "first ({}) must not come after last ({})",
            first, last
        )));
    }

    let span = last - first;

    if span > Duration::days(3650) {
        return Err(AppError::BadRequest(format!(
            "date span too long ({})",
            span
        )));
    }

    let connection = pool.get()?;

    let rows: Vec<types::day::Day> = days
        .filter(menu_id.eq(menu.into_inner()).and(date.between(first, last)))
        .load::<models::day::Day>(&connection)?
        .into_iter()
        .map(|d| d.into())
        .collect();

    let res = HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::MaxAge(3600)]))
        .json(rows);

    Ok(res)
}

/// Configure menu routes.
pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(list_menus_route)))
        .service(query_menu_route)
        .service(list_days_route);
}
