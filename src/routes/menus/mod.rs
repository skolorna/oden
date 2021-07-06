use actix_web::{
    get,
    http::header::{CacheControl, CacheDirective},
    web::{self, ServiceConfig},
    HttpResponse,
};

use crate::{
    errors::Result,
    menus::{id::MenuID, list_days, list_menus, query_menu},
};

/// Route for listing menus.
async fn list_menus_route() -> Result<HttpResponse> {
    let menus = list_menus().await?;
    let res = HttpResponse::Ok()
        .set(CacheControl(vec![CacheDirective::MaxAge(86_400)]))
        .json(menus);

    Ok(res)
}

#[get("{menu_id}")]
async fn query_menu_route(web::Path(menu_id): web::Path<MenuID>) -> Result<HttpResponse> {
    let menu = query_menu(&menu_id).await?;

    Ok(HttpResponse::Ok().json(menu))
}

/// Route for listing days.
#[get("{menu_id}/days")]
async fn list_days_route(web::Path(menu_id): web::Path<MenuID>) -> Result<HttpResponse> {
    let days = list_days(&menu_id).await?;

    Ok(HttpResponse::Ok().json(days))
}

/// Configure menu routes.
pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(list_menus_route)))
        .service(query_menu_route)
        .service(list_days_route);
}
