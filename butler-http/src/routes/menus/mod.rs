use actix_web::{
    get,
    http::header::{CacheControl, CacheDirective},
    web::{self, ServiceConfig},
    HttpResponse,
};
use butler::menus::id::MenuID;
use chrono::NaiveDate;
use serde::Deserialize;

use butler::{
    errors::ButlerResult,
    menus::list_menus,
};

use crate::{errors::AppResult, routes::swr};

/// Route for listing menus.
async fn list_menus_route() -> AppResult<HttpResponse> {
    let menus = list_menus(10).await?;
    let res = HttpResponse::Ok()
        .set(CacheControl(vec![
            CacheDirective::MaxAge(604_800), // 7 days
            swr(2_419_200),                  // 28 days
        ]))
        .json(menus);

    Ok(res)
}

#[get("{menu_id}")]
async fn query_menu_route(web::Path(menu_id): web::Path<MenuID>) -> AppResult<HttpResponse> {
    let menu = query_menu(&menu_id).await?;
    let res = HttpResponse::Ok()
        .set(CacheControl(vec![
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
    web::Path(menu_id): web::Path<MenuID>,
    query: web::Query<ListDaysRouteQuery>,
) -> AppResult<HttpResponse> {
    let query = ListDaysQuery::new(menu_id, query.first, query.last)?;
    let days = list_days(&query).await?;
    let res = HttpResponse::Ok()
        .set(CacheControl(vec![CacheDirective::MaxAge(86_400)]))
        .json(days);

    Ok(res)
}

/// Configure menu routes.
pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(list_menus_route)))
        .service(query_menu_route)
        .service(list_days_route);
}
