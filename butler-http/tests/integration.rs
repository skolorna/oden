use std::str::FromStr;

use actix_web::{
    http::StatusCode,
    test::{call_service, init_service, read_body_json, TestRequest},
};
use butler_lib::{
    menus::{day::Day, id::MenuId, supplier::Supplier, Menu},
    util::is_sorted,
};
use butler_http::create_app;

/// Perform a GET request.
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
async fn server_header() {
    let mut app = init_service(create_app!()).await;
    let resp = get!(app, "/thisshoulddefinitelyreturn404");
    let header = resp.headers().get("server").unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        header.to_str().unwrap(),
        format!("butler-http/{}", env!("CARGO_PKG_VERSION"))
    );
}

#[actix_rt::test]
async fn list_menus() {
    let mut app = init_service(create_app!()).await;
    let resp = get!(app, "/menus");
    assert_eq!(resp.status(), StatusCode::OK);

    let menus: Vec<Menu> = read_body_json(resp).await;
    assert!(menus.len() > 5000);

    // Sanity check
    assert!(Supplier::from_str("invalid provider id").is_err());

    for menu in menus {
        assert!(Supplier::from_str(&menu.supplier().id).is_ok());
    }
}

#[actix_rt::test]
async fn query_menu() {
    let mut app = init_service(create_app!()).await;

    {
        let resp = get!(app, "/menus/skolmaten.85957002");
        assert_eq!(resp.status(), StatusCode::OK);

        let menu: Menu = read_body_json(resp).await;

        assert_eq!(menu.title(), "P A Fogelströms gymnasium, Stockholms stad");
        assert_eq!(
            Supplier::from_str(&menu.supplier().id).unwrap(),
            Supplier::Skolmaten
        );
        assert_eq!(
            menu.id().clone(),
            MenuId::new(Supplier::Skolmaten, "85957002".to_owned())
        );
    }

    {
        let resp = get!(app, "/menus/skolmaten.123");
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    {
        let resp = get!(app, "/menus/mpi.c3c75403-6811-400a-96f8-a0e400c020ba");
        assert_eq!(resp.status(), StatusCode::OK);
        let menu: Menu = read_body_json(resp).await;
        assert_eq!(
            menu.title(),
            "Södra Latins Gymnasium, Stockholm, Fraiche Catering"
        );
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
