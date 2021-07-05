use actix_web::{
    web::{self, ServiceConfig},
    HttpResponse,
};

use crate::menus::{providers::skolmaten::Skolmaten, Provider};

async fn list_menus() -> HttpResponse {
    // let menus = vec!["a", "b", "c", "d"];

    let menus = Skolmaten::list_menus().await;

    HttpResponse::Ok().json(menus)
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(list_menus)));
}
