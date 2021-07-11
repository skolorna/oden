use std::str::FromStr;

use actix_web::{
    http::StatusCode,
    test::{call_service, init_service, read_body_json, TestRequest},
};
use menu_proxy::{
    create_app,
    menus::{day::Day, id::MenuID, provider::Provider, Menu},
    util::is_sorted,
};

/// Perform a GET request.
#[macro_export]
macro_rules! get {
    ($app:expr, $uri:expr) => {{
        let req = TestRequest::with_uri($uri).to_request();
        call_service(&mut $app, req).await
    }};
}

#[actix_rt::test]
async fn health_ok() {
    let mut app = init_service(create_app!()).await;
    let resp = get!(app, "/health");
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn list_menus() {
    let mut app = init_service(create_app!()).await;
    let resp = get!(app, "/menus");
    assert_eq!(resp.status(), StatusCode::OK);

    let menus: Vec<Menu> = read_body_json(resp).await;
    assert!(menus.len() > 5000);

    // Sanity check
    assert!(Provider::from_str("invalid provider id").is_err());

    for menu in menus {
        assert!(Provider::from_str(&menu.provider.id).is_ok());
    }
}

#[actix_rt::test]
async fn query_menu() {
    let mut app = init_service(create_app!()).await;

    {
        let resp = get!(app, "/menus/skolmaten.85957002");
        assert_eq!(resp.status(), StatusCode::OK);

        let menu: Menu = read_body_json(resp).await;

        assert_eq!(menu.title, "P A Fogelströms gymnasium, Stockholms stad");
        assert_eq!(
            Provider::from_str(&menu.provider.id).unwrap(),
            Provider::Skolmaten
        );
        assert_eq!(
            menu.id,
            MenuID::new(Provider::Skolmaten, "85957002".to_owned())
        );
    }

    {
        let resp = get!(app, "/menus/skolmaten.123");
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}

#[actix_rt::test]
async fn list_days() {
    let mut app = init_service(create_app!()).await;

    {
        let resp = get!(app, "/menus/skolmaten.4791333780717568/days");
        assert_eq!(resp.status(), StatusCode::OK);

        let days: Vec<Day> = read_body_json(resp).await;

        assert!(is_sorted(&days));
    }

    let bad_querys = vec![
        "first=1970-01-01&last=2020-12-31",
        "first=2020&last=2021-01-01",
        "first=2021-06-01&last=2021-05-31",
        "first=2019-02-29&last=2019-03-31", // 2019 is not a leap year
    ];

    for query in bad_querys {
        let resp = get!(
            app,
            &format!("/menus/skolmaten.4791333780717568/days?{}", query)
        );
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
