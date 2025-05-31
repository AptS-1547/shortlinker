// 本测试存在部分问题，需要等待解决

use actix_web::{middleware::from_fn, test, web, App};
use serde_json::json;
use std::env;
use tempfile::TempDir;

// 导入实际的项目代码
use shortlinker::admin::*;
use shortlinker::handle_redirect;

// 添加身份验证中间件
use shortlinker::admin::auth_middleware;

fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let links_file = temp_dir.path().join("test_links.json");
    env::set_var("STORAGE_BACKEND", "file");
    env::set_var("LINKS_FILE", links_file.to_str().unwrap());
    temp_dir
}

fn cleanup_env() {
    // 清理可能影响测试的环境变量
    env::remove_var("ADMIN_TOKEN");
    env::remove_var("DEFAULT_URL");
    env::remove_var("LINKS_FILE");
    env::remove_var("STORAGE_BACKEND");
}

#[actix_rt::test]
async fn test_shortlink_redirect_empty_path() {
    cleanup_env();
    let _temp_dir = setup_test_env();
    env::set_var("DEFAULT_URL", "https://test.example.com");

    let app = test::init_service(App::new().service(handle_redirect)).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 307);
    let location = resp.headers().get("Location").unwrap().to_str().unwrap();
    assert_eq!(location, "https://test.example.com");

    cleanup_env();
}

#[actix_rt::test]
async fn test_shortlink_redirect_existing_code() {
    cleanup_env();
    let _temp_dir = setup_test_env();

    // 直接使用FileStorage而不是全局STORAGE
    use shortlinker::storages::{file::FileStorage, ShortLink, Storage};
    let storage = FileStorage::new();

    // 创建一个测试链接
    let test_link = ShortLink {
        code: "github".to_string(),
        target: "https://github.com".to_string(),
        created_at: chrono::Utc::now(),
        expires_at: None,
    };
    storage.set(test_link).await.unwrap();

    let app = test::init_service(App::new().service(handle_redirect)).await;

    let req = test::TestRequest::get().uri("/github").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 307);
    let location = resp.headers().get("Location").unwrap().to_str().unwrap();
    assert_eq!(location, "https://github.com");

    // 清理测试数据
    storage.remove("github").await.unwrap();
    cleanup_env();
}

#[actix_rt::test]
async fn test_shortlink_not_found() {
    cleanup_env();
    let _temp_dir = setup_test_env();
    // 不设置 DEFAULT_URL，应该返回 404

    let app = test::init_service(App::new().service(handle_redirect)).await;

    let req = test::TestRequest::get().uri("/github").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 404);
    cleanup_env();
}

#[actix_rt::test]
async fn test_admin_api_disabled() {
    cleanup_env();
    let _temp_dir = setup_test_env();
    // 确保 ADMIN_TOKEN 为空
    env::set_var("ADMIN_TOKEN", "");

    // 验证环境变量设置
    assert_eq!(env::var("ADMIN_TOKEN").unwrap_or_default(), "");

    let app = test::init_service(
        App::new().service(
            web::scope("/admin")
                .wrap(from_fn(auth_middleware))
                .service(get_all_links)
                .service(post_link)
                .service(get_link)
                .service(delete_link)
                .service(update_link),
        ),
    )
    .await;

    let req = test::TestRequest::get().uri("/admin/link").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
    cleanup_env();
}

// 注意：以下管理员API测试需要特别处理环境变量
// 由于actix_rt的线程隔离机制，测试线程中设置的环境变量无法被中间件线程读取
// 解决方案：在auth_middleware中手动添加env::set_var来确保ADMIN_TOKEN可以被正确读取
//
// 问题原因：
// 1. actix_rt为每个请求创建独立的执行上下文
// 2. 环境变量在不同线程间可能不共享
// 3. 中间件运行在与测试设置不同的上下文中
//
// 临时解决方案：
// 在shortlinker/src/admin.rs的auth_middleware函数开头添加：
// env::set_var("ADMIN_TOKEN", "test_token");
#[actix_rt::test]
async fn test_admin_api_auth_required() {
    cleanup_env();
    let _temp_dir = setup_test_env();
    env::set_var("ADMIN_TOKEN", "test_token");

    // 验证环境变量设置
    assert_eq!(env::var("ADMIN_TOKEN").unwrap(), "test_token");

    let app = test::init_service(
        App::new().service(
            web::scope("/admin")
                .wrap(from_fn(auth_middleware))
                .service(get_all_links)
                .service(post_link)
                .service(get_link)
                .service(delete_link)
                .service(update_link),
        ),
    )
    .await;

    // 测试无Authorization header
    let req = test::TestRequest::get().uri("/admin/link").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // 测试错误的token
    let req = test::TestRequest::get()
        .uri("/admin/link")
        .insert_header(("Authorization", "Bearer wrong_token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // 测试正确的token
    let req = test::TestRequest::get()
        .uri("/admin/link")
        .insert_header(("Authorization", "Bearer test_token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    cleanup_env();
}

// 注意：此测试同样受到actix_rt线程隔离影响
// 需要在auth_middleware中手动设置ADMIN_TOKEN环境变量
// 否则中间件无法读取到测试中设置的环境变量值
#[actix_rt::test]
async fn test_admin_api_create_link() {
    cleanup_env();
    let _temp_dir = setup_test_env();
    env::set_var("ADMIN_TOKEN", "test_token");

    let app = test::init_service(
        App::new().service(
            web::scope("/admin")
                .wrap(from_fn(auth_middleware))
                .service(post_link)
                .service(get_link),
        ),
    )
    .await;

    // 创建新链接
    let new_link = json!({
        "code": "test",
        "target": "https://example.com",
        "expires_at": null
    });

    let req = test::TestRequest::post()
        .uri("/admin/link")
        .insert_header(("Authorization", "Bearer test_token"))
        .insert_header(("Content-Type", "application/json"))
        .set_json(&new_link)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // 获取刚创建的链接
    let req = test::TestRequest::get()
        .uri("/admin/link/test")
        .insert_header(("Authorization", "Bearer test_token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    cleanup_env();
}

// 注意：此测试同样受到actix_rt线程隔离影响
// 需要在auth_middleware中手动设置ADMIN_TOKEN环境变量
#[actix_rt::test]
async fn test_admin_api_delete_link() {
    cleanup_env();
    let _temp_dir = setup_test_env();
    env::set_var("ADMIN_TOKEN", "test_token");

    let app = test::init_service(
        App::new().service(
            web::scope("/admin")
                .wrap(from_fn(auth_middleware))
                .service(post_link)
                .service(delete_link)
                .service(get_link),
        ),
    )
    .await;

    // 先创建一个链接
    let new_link = json!({
        "code": "delete_test",
        "target": "https://example.com",
        "expires_at": null
    });

    let req = test::TestRequest::post()
        .uri("/admin/link")
        .insert_header(("Authorization", "Bearer test_token"))
        .insert_header(("Content-Type", "application/json"))
        .set_json(&new_link)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // 删除链接
    let req = test::TestRequest::delete()
        .uri("/admin/link/delete_test")
        .insert_header(("Authorization", "Bearer test_token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // 验证链接已被删除
    let req = test::TestRequest::get()
        .uri("/admin/link/delete_test")
        .insert_header(("Authorization", "Bearer test_token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    cleanup_env();
}

// 注意：此测试同样受到actix_rt线程隔离影响
// 需要在auth_middleware中手动设置ADMIN_TOKEN环境变量
#[actix_rt::test]
async fn test_admin_api_update_link() {
    cleanup_env();
    let _temp_dir = setup_test_env();
    env::set_var("ADMIN_TOKEN", "test_token");

    let app = test::init_service(
        App::new().service(
            web::scope("/admin")
                .wrap(from_fn(auth_middleware))
                .service(post_link)
                .service(update_link)
                .service(get_link),
        ),
    )
    .await;

    // 先创建一个链接
    let new_link = json!({
        "code": "update_test",
        "target": "https://original.com",
        "expires_at": null
    });

    let req = test::TestRequest::post()
        .uri("/admin/link")
        .insert_header(("Authorization", "Bearer test_token"))
        .insert_header(("Content-Type", "application/json"))
        .set_json(&new_link)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // 更新链接
    let updated_link = json!({
        "code": "update_test",
        "target": "https://updated.com",
        "expires_at": null
    });

    let req = test::TestRequest::put()
        .uri("/admin/link/update_test")
        .insert_header(("Authorization", "Bearer test_token"))
        .insert_header(("Content-Type", "application/json"))
        .set_json(&updated_link)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // 验证链接已被更新
    let req = test::TestRequest::get()
        .uri("/admin/link/update_test")
        .insert_header(("Authorization", "Bearer test_token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["target_url"], "https://updated.com");

    cleanup_env();
}

#[actix_rt::test]
async fn test_shortlink_with_actual_storage() {
    cleanup_env();
    let _temp_dir = setup_test_env();

    // 直接使用FileStorage而不是全局STORAGE
    use shortlinker::storages::{file::FileStorage, ShortLink, Storage};
    let storage = FileStorage::new();

    // 创建一个测试链接
    let test_link = ShortLink {
        code: "testcode".to_string(),
        target: "https://actual-test.com".to_string(),
        created_at: chrono::Utc::now(),
        expires_at: None,
    };

    // 保存到实际存储
    storage.set(test_link).await.unwrap();

    let app = test::init_service(App::new().service(handle_redirect)).await;

    // 测试重定向
    let req = test::TestRequest::get().uri("/testcode").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 307);
    let location = resp.headers().get("Location").unwrap().to_str().unwrap();
    assert_eq!(location, "https://actual-test.com");

    // 清理测试数据
    storage.remove("testcode").await.unwrap();
    cleanup_env();
}

#[actix_rt::test]
async fn test_expired_link() {
    cleanup_env();
    let _temp_dir = setup_test_env();

    // 直接使用FileStorage而不是全局STORAGE
    use shortlinker::storages::{file::FileStorage, ShortLink, Storage};
    let storage = FileStorage::new();

    // 创建一个过期的链接
    let expired_link = ShortLink {
        code: "expired".to_string(),
        target: "https://expired.com".to_string(),
        created_at: chrono::Utc::now() - chrono::Duration::hours(2),
        expires_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
    };

    // 保存到实际存储
    storage.set(expired_link).await.unwrap();

    let app = test::init_service(App::new().service(handle_redirect)).await;

    // 测试过期链接应该返回404
    let req = test::TestRequest::get().uri("/expired").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 404);

    // 清理测试数据
    storage.remove("expired").await.unwrap();
    cleanup_env();
}
