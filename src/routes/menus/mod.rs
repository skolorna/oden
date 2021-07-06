use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse,
};

use crate::{
    errors::Result,
    menus::{providers::skolmaten::Skolmaten, Provider},
};

/// Route for listing menus.
async fn list_menus() -> Result<HttpResponse> {
    // let menus = vec!["a", "b", "c", "d"];

    let menus = Skolmaten::list_menus().await?;

    Ok(HttpResponse::Ok().json(menus))
}

#[get("{menu_id}")]
async fn query_menu(web::Path(menu_id): web::Path<String>) -> Result<HttpResponse> {
    let menu = Skolmaten::query_menu(&menu_id).await?;

    Ok(HttpResponse::Ok().json(menu))
}

/// Route for listing days.
#[get("{menu_id}/days")]
async fn list_days(web::Path(menu_id): web::Path<String>) -> Result<HttpResponse> {
    let days = Skolmaten::list_days(&menu_id).await?;

    Ok(HttpResponse::Ok().json(days))
}

/// Configure menu routes.
pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(list_menus)))
        .service(query_menu)
        .service(list_days);
}
