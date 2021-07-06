use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse,
};

use crate::menus::{providers::skolmaten::Skolmaten, Provider};

/// Route for listing menus.
async fn list_menus() -> HttpResponse {
    // let menus = vec!["a", "b", "c", "d"];

    let menus = Skolmaten::list_menus().await;

    HttpResponse::Ok().json(menus)
}

/// Route for listing days.
#[get("{menu_id}/days")]
async fn list_days(web::Path(menu_id): web::Path<String>) -> HttpResponse {
    let days = Skolmaten::list_days(menu_id).await;

    HttpResponse::Ok().json(days)
}

/// Configure menu routes.
pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(list_menus)))
        .service(list_days);
}
