//! Middleware tests
//!
//! Tests for AdminAuth and CsrfGuard middleware.
//! Replaces the old middleware_tests.rs.disabled.

use actix_web::http::{Method, StatusCode};
use actix_web::test::{self, TestRequest};
use actix_web::{App, HttpResponse, web};
use std::sync::Arc;

use shortlinker::api::middleware::{AdminAuth, CsrfGuard};
use shortlinker::config::init_config;
use shortlinker::config::runtime_config::init_runtime_config;
use shortlinker::metrics_core::NoopMetrics;
use shortlinker::storage::backend::{connect_sqlite, run_migrations};

use std::sync::Once;
use tempfile::TempDir;

// =============================================================================
// Test Setup
// =============================================================================

static INIT: Once = Once::new();
static TEST_DIR: std::sync::OnceLock<TempDir> = std::sync::OnceLock::new();
static RT_INIT: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

fn init_static_config() {
    INIT.call_once(|| {
        init_config();
    });
}

/// Initialize RuntimeConfig with a real SQLite database.
/// Must be called before any middleware test.
async fn init_test_runtime_config() {
    init_static_config();

    RT_INIT
        .get_or_init(|| async {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let db_path = temp_dir.path().join("middleware_test.db");
            let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

            let db = connect_sqlite(&db_url)
                .await
                .expect("Failed to connect to SQLite");
            run_migrations(&db).await.expect("Failed to run migrations");
            init_runtime_config(db)
                .await
                .expect("Failed to init runtime config");

            // Store TempDir in static to keep DB alive
            let _ = TEST_DIR.set(temp_dir);
        })
        .await;
}

/// Generate a valid JWT access token for testing.
fn generate_test_token() -> String {
    use shortlinker::api::jwt::get_jwt_service;
    get_jwt_service()
        .generate_access_token()
        .expect("Failed to generate test token")
}

/// Simple handler for testing middleware
async fn ok_handler() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

// =============================================================================
// AdminAuth Tests
// =============================================================================

#[tokio::test]
async fn test_admin_auth_missing_token_returns_404() {
    init_test_runtime_config().await;

    // Explicitly clear admin token
    let rt = shortlinker::config::get_runtime_config();
    let _ = rt.set("api.admin_token", "").await;

    // When admin token is empty, middleware returns 404
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(
                NoopMetrics::arc() as Arc<dyn shortlinker::metrics_core::MetricsRecorder>
            ))
            .service(
                web::scope("/admin")
                    .wrap(AdminAuth)
                    .route("/v1/test", web::get().to(ok_handler)),
            ),
    )
    .await;

    let req = TestRequest::get().uri("/admin/v1/test").to_request();
    let resp = test::call_service(&app, req).await;

    // Restore admin token to prevent race conditions with parallel tests
    let _ = rt.set("api.admin_token", "test-secret-token").await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_admin_auth_allows_login_endpoint() {
    init_test_runtime_config().await;

    // Set admin token so middleware doesn't return 404
    let rt = shortlinker::config::get_runtime_config();
    let _ = rt.set("api.admin_token", "test-secret-token").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(
                NoopMetrics::arc() as Arc<dyn shortlinker::metrics_core::MetricsRecorder>
            ))
            .service(
                web::scope("/admin")
                    .wrap(AdminAuth)
                    .route("/v1/auth/login", web::post().to(ok_handler)),
            ),
    )
    .await;

    let req = TestRequest::post().uri("/admin/v1/auth/login").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_auth_allows_refresh_endpoint() {
    init_test_runtime_config().await;

    let rt = shortlinker::config::get_runtime_config();
    let _ = rt.set("api.admin_token", "test-secret-token").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(
                NoopMetrics::arc() as Arc<dyn shortlinker::metrics_core::MetricsRecorder>
            ))
            .service(
                web::scope("/admin")
                    .wrap(AdminAuth)
                    .route("/v1/auth/refresh", web::post().to(ok_handler)),
            ),
    )
    .await;

    let req = TestRequest::post()
        .uri("/admin/v1/auth/refresh")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_auth_allows_logout_endpoint() {
    init_test_runtime_config().await;

    let rt = shortlinker::config::get_runtime_config();
    let _ = rt.set("api.admin_token", "test-secret-token").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(
                NoopMetrics::arc() as Arc<dyn shortlinker::metrics_core::MetricsRecorder>
            ))
            .service(
                web::scope("/admin")
                    .wrap(AdminAuth)
                    .route("/v1/auth/logout", web::post().to(ok_handler)),
            ),
    )
    .await;

    let req = TestRequest::post()
        .uri("/admin/v1/auth/logout")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_auth_handles_options() {
    init_test_runtime_config().await;

    let rt = shortlinker::config::get_runtime_config();
    let _ = rt.set("api.admin_token", "test-secret-token").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(
                NoopMetrics::arc() as Arc<dyn shortlinker::metrics_core::MetricsRecorder>
            ))
            .service(
                web::scope("/admin")
                    .wrap(AdminAuth)
                    .route("/v1/test", web::method(Method::OPTIONS).to(ok_handler)),
            ),
    )
    .await;

    let req = TestRequest::default()
        .method(Method::OPTIONS)
        .uri("/admin/v1/test")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_admin_auth_rejects_no_token() {
    init_test_runtime_config().await;

    let rt = shortlinker::config::get_runtime_config();
    let _ = rt.set("api.admin_token", "test-secret-token").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(
                NoopMetrics::arc() as Arc<dyn shortlinker::metrics_core::MetricsRecorder>
            ))
            .service(
                web::scope("/admin")
                    .wrap(AdminAuth)
                    .route("/v1/test", web::get().to(ok_handler)),
            ),
    )
    .await;

    // Request without any auth token
    let req = TestRequest::get().uri("/admin/v1/test").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_auth_valid_bearer_token() {
    init_test_runtime_config().await;

    let rt = shortlinker::config::get_runtime_config();
    let _ = rt.set("api.admin_token", "test-secret-token").await;

    let token = generate_test_token();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(
                NoopMetrics::arc() as Arc<dyn shortlinker::metrics_core::MetricsRecorder>
            ))
            .service(
                web::scope("/admin")
                    .wrap(AdminAuth)
                    .route("/v1/test", web::get().to(ok_handler)),
            ),
    )
    .await;

    let req = TestRequest::get()
        .uri("/admin/v1/test")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_auth_invalid_bearer_token() {
    init_test_runtime_config().await;

    let rt = shortlinker::config::get_runtime_config();
    let _ = rt.set("api.admin_token", "test-secret-token").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(
                NoopMetrics::arc() as Arc<dyn shortlinker::metrics_core::MetricsRecorder>
            ))
            .service(
                web::scope("/admin")
                    .wrap(AdminAuth)
                    .route("/v1/test", web::get().to(ok_handler)),
            ),
    )
    .await;

    let req = TestRequest::get()
        .uri("/admin/v1/test")
        .insert_header(("Authorization", "Bearer invalid-token-here"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// =============================================================================
// CSRF Tests
// =============================================================================

#[tokio::test]
async fn test_csrf_skips_safe_methods() {
    init_test_runtime_config().await;

    let rt = shortlinker::config::get_runtime_config();
    let _ = rt.set("api.admin_token", "test-secret-token").await;

    // CsrfGuard should skip GET requests
    let app = test::init_service(
        App::new().service(
            web::scope("/admin")
                .wrap(CsrfGuard)
                .route("/v1/test", web::get().to(ok_handler)),
        ),
    )
    .await;

    let req = TestRequest::get().uri("/admin/v1/test").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_csrf_skips_auth_endpoints() {
    init_test_runtime_config().await;

    let rt = shortlinker::config::get_runtime_config();
    let _ = rt.set("api.admin_token", "test-secret-token").await;

    // CsrfGuard should skip login endpoint even for POST
    let app = test::init_service(
        App::new().service(
            web::scope("/admin")
                .wrap(CsrfGuard)
                .route("/v1/auth/login", web::post().to(ok_handler)),
        ),
    )
    .await;

    let req = TestRequest::post().uri("/admin/v1/auth/login").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_csrf_rejects_post_without_token() {
    init_test_runtime_config().await;

    let rt = shortlinker::config::get_runtime_config();
    let _ = rt.set("api.admin_token", "test-secret-token").await;

    // POST to non-auth endpoint without CSRF token should be rejected
    let app = test::init_service(
        App::new().service(
            web::scope("/admin")
                .wrap(CsrfGuard)
                .route("/v1/links", web::post().to(ok_handler)),
        ),
    )
    .await;

    let req = TestRequest::post().uri("/admin/v1/links").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_csrf_rejects_mismatched_tokens() {
    init_test_runtime_config().await;

    let rt = shortlinker::config::get_runtime_config();
    let _ = rt.set("api.admin_token", "test-secret-token").await;

    let app = test::init_service(
        App::new().service(
            web::scope("/admin")
                .wrap(CsrfGuard)
                .route("/v1/links", web::post().to(ok_handler)),
        ),
    )
    .await;

    // Cookie and header have different CSRF tokens
    let req = TestRequest::post()
        .uri("/admin/v1/links")
        .cookie(actix_web::cookie::Cookie::new("csrf_token", "token-a"))
        .insert_header(("X-CSRF-Token", "token-b"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_csrf_accepts_matching_tokens() {
    init_test_runtime_config().await;

    let rt = shortlinker::config::get_runtime_config();
    let _ = rt.set("api.admin_token", "test-secret-token").await;

    let app = test::init_service(
        App::new().service(
            web::scope("/admin")
                .wrap(CsrfGuard)
                .route("/v1/links", web::post().to(ok_handler)),
        ),
    )
    .await;

    let csrf_token = "valid-csrf-token-12345";
    let req = TestRequest::post()
        .uri("/admin/v1/links")
        .cookie(actix_web::cookie::Cookie::new("csrf_token", csrf_token))
        .insert_header(("X-CSRF-Token", csrf_token))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
