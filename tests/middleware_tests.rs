use actix_web::{http::Method, http::StatusCode, test, web, App, HttpResponse};
use shortlinker::middleware::AdminAuth;
use std::sync::Mutex;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[actix_web::test]
async fn admin_auth_missing_token_returns_404() {
    let _guard = ENV_MUTEX.lock().unwrap();
    std::env::set_var("ADMIN_TOKEN", "");
    std::env::set_var("ADMIN_ROUTE_PREFIX", "/admin");

    let app = test::init_service(App::new().wrap(AdminAuth).route(
        "/admin/test",
        web::get().to(|| async { HttpResponse::Ok() }),
    ))
    .await;

    let req = test::TestRequest::get().uri("/admin/test").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn admin_auth_allows_login_without_token() {
    let _guard = ENV_MUTEX.lock().unwrap();
    std::env::set_var("ADMIN_TOKEN", "secret");
    std::env::set_var("ADMIN_ROUTE_PREFIX", "/admin");

    let app = test::init_service(App::new().wrap(AdminAuth).route(
        "/admin/auth/login",
        web::get().to(|| async { HttpResponse::Ok() }),
    ))
    .await;

    let req = test::TestRequest::get()
        .uri("/admin/auth/login")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn admin_auth_validates_token() {
    let _guard = ENV_MUTEX.lock().unwrap();
    std::env::set_var("ADMIN_TOKEN", "secret");
    std::env::set_var("ADMIN_ROUTE_PREFIX", "/admin");

    let app = test::init_service(App::new().wrap(AdminAuth).route(
        "/admin/protected",
        web::get().to(|| async { HttpResponse::Ok() }),
    ))
    .await;

    let req = test::TestRequest::get()
        .uri("/admin/protected")
        .insert_header(("Authorization", "Bearer secret"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let req_invalid = test::TestRequest::get()
        .uri("/admin/protected")
        .insert_header(("Authorization", "Bearer wrong"))
        .to_request();
    let resp = test::call_service(&app, req_invalid).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn admin_auth_handles_options() {
    let _guard = ENV_MUTEX.lock().unwrap();
    std::env::set_var("ADMIN_TOKEN", "secret");
    std::env::set_var("ADMIN_ROUTE_PREFIX", "/admin");

    let app = test::init_service(App::new().wrap(AdminAuth).route(
        "/admin/test",
        web::get().to(|| async { HttpResponse::Ok() }),
    ))
    .await;

    let req = test::TestRequest::default()
        .method(Method::OPTIONS)
        .uri("/admin/test")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}
