use std::str::FromStr;

use actix_web::{http::StatusCode, test};
use menu_proxy::{
    create_app,
    menus::{provider::Provider, Menu},
};

#[actix_rt::test]
async fn health_ok() {
    let mut app = test::init_service(create_app!()).await;

    let req = test::TestRequest::with_uri("/health").to_request();
    let resp = test::call_service(&mut app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn list_menus() {
    let mut app = test::init_service(create_app!()).await;

    let req = test::TestRequest::with_uri("/menus").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let menus: Vec<Menu> = test::read_body_json(resp).await;
    assert!(menus.len() > 5000);

    for menu in menus {
        assert!(Provider::from_str(menu.provider_id()).is_ok());
    }
}
