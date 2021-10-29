use actix_web::{
    get,
    http::header::{CacheControl, CacheDirective},
    web::{self, ServiceConfig},
    HttpResponse,
};
use butler::menus::{id::MenuId, list_days, query_menu};
use chrono::{Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Europe::Stockholm;
use serde::Deserialize;

use butler::menus::list_menus;

use crate::{
    errors::{AppError, AppResult},
    routes::swr,
};

/// Route for listing menus.
async fn list_menus_route() -> AppResult<HttpResponse> {
    let menus = list_menus(10).await?;
    let res = HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::MaxAge(604_800), // 7 days
            swr(2_419_200),                  // 28 days
        ]))
        .json(menus);

    Ok(res)
}

#[get("{menu_id}")]
async fn query_menu_route(menu_id: web::Path<MenuId>) -> AppResult<HttpResponse> {
    let menu = query_menu(&menu_id).await?;
    let res = HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::MaxAge(604_800), // 7 days
            swr(2_419_200),                  // 28 days
        ]))
        .json(menu);

    Ok(res)
}

#[derive(Deserialize)]
struct ListDaysRouteQuery {
    first: Option<NaiveDate>,
    last: Option<NaiveDate>,
}

/// Route for listing days.
#[get("{menu_id}/days")]
async fn list_days_route(
    menu_id: web::Path<MenuId>,
    query: web::Query<ListDaysRouteQuery>,
) -> AppResult<HttpResponse> {
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

    let days = list_days(&menu_id, first, last).await?;
    let res = HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::MaxAge(86_400)]))
        .json(days);

    Ok(res)
}

/// Configure menu routes.
pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(list_menus_route)))
        .service(query_menu_route)
        .service(list_days_route);
}
