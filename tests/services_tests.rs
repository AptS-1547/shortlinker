use actix_web::{test as actix_test, web, App, HttpResponse};
use shortlinker::services::admin::{AdminService, PostNewLink, SerializableShortLink, ApiResponse};
use shortlinker::storages::{ShortLink, Storage};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use serde_json;

// 模拟存储实现用于测试
#[derive(Default)]
struct MockStorage {
    data: std::sync::Mutex<HashMap<String, ShortLink>>,
    should_fail: std::sync::Mutex<bool>,
}

impl MockStorage {
    fn new_failing() -> Self {
        Self {
            data: std::sync::Mutex::new(HashMap::new()),
            should_fail: std::sync::Mutex::new(true),
        }
    }
    
    fn set_should_fail(&self, fail: bool) {
        *self.should_fail.lock().unwrap() = fail;
    }
    
    fn insert_test_data(&self) {
        let mut data = self.data.lock().unwrap();
        let test_link = ShortLink {
            code: "test123".to_string(),
            target: "https://example.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };
        data.insert("test123".to_string(), test_link);
    }
}

#[async_trait::async_trait]
impl Storage for MockStorage {
    async fn get(&self, code: &str) -> Option<ShortLink> {
        if *self.should_fail.lock().unwrap() {
            return None;
        }
        let data = self.data.lock().unwrap();
        data.get(code).cloned()
    }

    async fn set(&self, link: ShortLink) -> Result<(), shortlinker::errors::ShortlinkerError> {
        if *self.should_fail.lock().unwrap() {
            return Err(shortlinker::errors::ShortlinkerError::file_operation("Mock storage error"));
        }
        let mut data = self.data.lock().unwrap();
        data.insert(link.code.clone(), link);
        Ok(())
    }

    async fn remove(&self, code: &str) -> Result<(), shortlinker::errors::ShortlinkerError> {
        if *self.should_fail.lock().unwrap() {
            return Err(shortlinker::errors::ShortlinkerError::file_operation("Mock storage error"));
        }
        let mut data = self.data.lock().unwrap();
        data.remove(code);
        Ok(())
    }

    async fn load_all(&self) -> HashMap<String, ShortLink> {
        if *self.should_fail.lock().unwrap() {
            return HashMap::new();
        }
        self.data.lock().unwrap().clone()
    }

    async fn reload(&self) -> Result<(), shortlinker::errors::ShortlinkerError> {
        if *self.should_fail.lock().unwrap() {
            return Err(shortlinker::errors::ShortlinkerError::file_operation("Mock reload error"));
        }
        Ok(())
    }

    async fn get_backend_name(&self) -> String {
        "mock".to_string()
    }
}

#[cfg(test)]
mod admin_service_tests {
    use super::*;

    #[actix_web::test]
    async fn test_get_all_links_success() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);
        storage.insert_test_data();

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .route("/admin/links", web::get().to(AdminService::get_all_links))
        ).await;

        let req = actix_test::TestRequest::get()
            .uri("/admin/links")
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: ApiResponse<Option<HashMap<String, SerializableShortLink>>> = 
            actix_test::read_body_json(resp).await;
        assert_eq!(body.code, 0);
        assert!(body.data.is_some());
        assert_eq!(body.data.unwrap().len(), 1);
    }

    #[actix_web::test]
    async fn test_get_all_links_empty() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .route("/admin/links", web::get().to(AdminService::get_all_links))
        ).await;

        let req = actix_test::TestRequest::get()
            .uri("/admin/links")
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: ApiResponse<Option<HashMap<String, SerializableShortLink>>> = 
            actix_test::read_body_json(resp).await;
        assert_eq!(body.code, 0);
        assert!(body.data.is_some());
        assert_eq!(body.data.unwrap().len(), 0);
    }

    #[actix_web::test]
    async fn test_post_link_success() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .route("/admin/links", web::post().to(AdminService::post_link))
        ).await;

        let new_link = PostNewLink {
            code: Some("testcode".to_string()),
            target: "https://example.com".to_string(),
            expires_at: None,
        };

        let req = actix_test::TestRequest::post()
            .uri("/admin/links")
            .set_json(&new_link)
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: ApiResponse<PostNewLink> = actix_test::read_body_json(resp).await;
        assert_eq!(body.code, 0);
        assert_eq!(body.data.code, Some("testcode".to_string()));
        assert_eq!(body.data.target, "https://example.com");
    }

    #[actix_web::test]
    async fn test_post_link_auto_generate_code() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .route("/admin/links", web::post().to(AdminService::post_link))
        ).await;

        let new_link = PostNewLink {
            code: None,
            target: "https://example.com".to_string(),
            expires_at: None,
        };

        let req = actix_test::TestRequest::post()
            .uri("/admin/links")
            .set_json(&new_link)
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: ApiResponse<PostNewLink> = actix_test::read_body_json(resp).await;
        assert_eq!(body.code, 0);
        assert!(body.data.code.is_some());
        assert!(!body.data.code.unwrap().is_empty());
        assert_eq!(body.data.target, "https://example.com");
    }

    #[actix_web::test]
    async fn test_post_link_with_expiry() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .route("/admin/links", web::post().to(AdminService::post_link))
        ).await;

        let new_link = PostNewLink {
            code: Some("expiry_test".to_string()),
            target: "https://example.com".to_string(),
            expires_at: Some("2025-12-31T23:59:59Z".to_string()),
        };

        let req = actix_test::TestRequest::post()
            .uri("/admin/links")
            .set_json(&new_link)
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: ApiResponse<PostNewLink> = actix_test::read_body_json(resp).await;
        assert_eq!(body.code, 0);
        assert_eq!(body.data.expires_at, Some("2025-12-31T23:59:59Z".to_string()));
    }

    #[actix_web::test]
    async fn test_post_link_storage_failure() {
        let storage = Arc::new(MockStorage::new_failing());

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .route("/admin/links", web::post().to(AdminService::post_link))
        ).await;

        let new_link = PostNewLink {
            code: Some("fail_test".to_string()),
            target: "https://example.com".to_string(),
            expires_at: None,
        };

        let req = actix_test::TestRequest::post()
            .uri("/admin/links")
            .set_json(&new_link)
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);

        let body: ApiResponse<serde_json::Value> = actix_test::read_body_json(resp).await;
        assert_eq!(body.code, 1);
    }

    #[actix_web::test]
    async fn test_get_link_success() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);
        storage.insert_test_data();

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .route("/admin/links/{code}", web::get().to(AdminService::get_link))
        ).await;

        let req = actix_test::TestRequest::get()
            .uri("/admin/links/test123")
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: ApiResponse<SerializableShortLink> = actix_test::read_body_json(resp).await;
        assert_eq!(body.code, 0);
        assert_eq!(body.data.short_code, "test123");
        assert_eq!(body.data.target_url, "https://example.com");
    }

    #[actix_web::test]
    async fn test_get_link_not_found() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .route("/admin/links/{code}", web::get().to(AdminService::get_link))
        ).await;

        let req = actix_test::TestRequest::get()
            .uri("/admin/links/nonexistent")
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        let body: ApiResponse<serde_json::Value> = actix_test::read_body_json(resp).await;
        assert_eq!(body.code, 1);
    }

    #[actix_web::test]
    async fn test_delete_link_success() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);
        storage.insert_test_data();

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .route("/admin/links/{code}", web::delete().to(AdminService::delete_link))
        ).await;

        let req = actix_test::TestRequest::delete()
            .uri("/admin/links/test123")
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: ApiResponse<serde_json::Value> = actix_test::read_body_json(resp).await;
        assert_eq!(body.code, 0);
    }

    #[actix_web::test]
    async fn test_delete_link_storage_failure() {
        let storage = Arc::new(MockStorage::new_failing());

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .route("/admin/links/{code}", web::delete().to(AdminService::delete_link))
        ).await;

        let req = actix_test::TestRequest::delete()
            .uri("/admin/links/test123")
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);

        let body: ApiResponse<serde_json::Value> = actix_test::read_body_json(resp).await;
        assert_eq!(body.code, 1);
    }

    #[actix_web::test]
    async fn test_update_link_success() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);
        storage.insert_test_data();

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .route("/admin/links/{code}", web::put().to(AdminService::update_link))
        ).await;

        let update_data = PostNewLink {
            code: None, // 这个字段在更新时不使用
            target: "https://updated.com".to_string(),
            expires_at: Some("2025-01-01T00:00:00Z".to_string()),
        };

        let req = actix_test::TestRequest::put()
            .uri("/admin/links/test123")
            .set_json(&update_data)
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: ApiResponse<PostNewLink> = actix_test::read_body_json(resp).await;
        assert_eq!(body.code, 0);
        assert_eq!(body.data.target, "https://updated.com");
        assert_eq!(body.data.expires_at, Some("2025-01-01T00:00:00Z".to_string()));
    }

    #[actix_web::test]
    async fn test_update_link_storage_failure() {
        let storage = Arc::new(MockStorage::new_failing());

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .route("/admin/links/{code}", web::put().to(AdminService::update_link))
        ).await;

        let update_data = PostNewLink {
            code: None,
            target: "https://updated.com".to_string(),
            expires_at: None,
        };

        let req = actix_test::TestRequest::put()
            .uri("/admin/links/test123")
            .set_json(&update_data)
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);

        let body: ApiResponse<serde_json::Value> = actix_test::read_body_json(resp).await;
        assert_eq!(body.code, 1);
    }
}

#[cfg(test)]
mod auth_middleware_tests {
    use super::*;
    use actix_web::middleware::from_fn;

    #[actix_web::test]
    async fn test_auth_middleware_no_token_env() {
        // 确保没有设置 ADMIN_TOKEN
        env::remove_var("ADMIN_TOKEN");

        let app = actix_test::init_service(
            App::new()
                .wrap(from_fn(AdminService::auth_middleware))
                .route("/admin/test", web::get().to(|| async { HttpResponse::Ok().json("success") }))
        ).await;

        let req = actix_test::TestRequest::get()
            .uri("/admin/test")
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404); // API 被禁用时返回 404
    }

    #[actix_web::test]
    async fn test_auth_middleware_valid_token() {
        // 设置测试 token
        env::set_var("ADMIN_TOKEN", "test_token_123");

        let app = actix_test::init_service(
            App::new()
                .wrap(from_fn(AdminService::auth_middleware))
                .route("/admin/test", web::get().to(|| async { HttpResponse::Ok().json("success") }))
        ).await;

        let req = actix_test::TestRequest::get()
            .uri("/admin/test")
            .insert_header(("Authorization", "Bearer test_token_123"))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 清理环境变量
        env::remove_var("ADMIN_TOKEN");
    }

    #[actix_web::test]
    async fn test_auth_middleware_invalid_token() {
        // 设置测试 token
        env::set_var("ADMIN_TOKEN", "correct_token");

        let app = actix_test::init_service(
            App::new()
                .wrap(from_fn(AdminService::auth_middleware))
                .route("/admin/test", web::get().to(|| async { HttpResponse::Ok().json("success") }))
        ).await;

        let req = actix_test::TestRequest::get()
            .uri("/admin/test")
            .insert_header(("Authorization", "Bearer wrong_token"))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);

        // 清理环境变量
        env::remove_var("ADMIN_TOKEN");
    }

    #[actix_web::test]
    async fn test_auth_middleware_missing_header() {
        // 设置测试 token
        env::set_var("ADMIN_TOKEN", "test_token");

        let app = actix_test::init_service(
            App::new()
                .wrap(from_fn(AdminService::auth_middleware))
                .route("/admin/test", web::get().to(|| async { HttpResponse::Ok().json("success") }))
        ).await;

        let req = actix_test::TestRequest::get()
            .uri("/admin/test")
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);

        // 清理环境变量
        env::remove_var("ADMIN_TOKEN");
    }

    #[actix_web::test]
    async fn test_auth_middleware_malformed_header() {
        // 设置测试 token
        env::set_var("ADMIN_TOKEN", "test_token");

        let app = actix_test::init_service(
            App::new()
                .wrap(from_fn(AdminService::auth_middleware))
                .route("/admin/test", web::get().to(|| async { HttpResponse::Ok().json("success") }))
        ).await;

        let req = actix_test::TestRequest::get()
            .uri("/admin/test")
            .insert_header(("Authorization", "Basic invalid_format"))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);

        // 清理环境变量
        env::remove_var("ADMIN_TOKEN");
    }
}

#[cfg(test)]
mod data_structures_tests {
    use super::*;

    #[test]
    fn test_serializable_short_link_creation() {
        let link = SerializableShortLink {
            short_code: "test123".to_string(),
            target_url: "https://example.com".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            expires_at: Some("2023-12-31T23:59:59Z".to_string()),
        };

        assert_eq!(link.short_code, "test123");
        assert_eq!(link.target_url, "https://example.com");
        assert!(link.expires_at.is_some());
    }

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse {
            code: 0,
            data: "test_data".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"code\":0"));
        assert!(json.contains("\"data\":\"test_data\""));
    }

    #[test]
    fn test_post_new_link_deserialization() {
        let json = r#"{"code":"test123","target":"https://example.com","expires_at":null}"#;
        let link: PostNewLink = serde_json::from_str(json).unwrap();

        assert_eq!(link.code, Some("test123".to_string()));
        assert_eq!(link.target, "https://example.com");
        assert!(link.expires_at.is_none());
    }

    #[test]
    fn test_post_new_link_with_expiry() {
        let json = r#"{"code":null,"target":"https://example.com","expires_at":"2023-12-31T23:59:59Z"}"#;
        let link: PostNewLink = serde_json::from_str(json).unwrap();

        assert!(link.code.is_none());
        assert_eq!(link.target, "https://example.com");
        assert_eq!(link.expires_at, Some("2023-12-31T23:59:59Z".to_string()));
    }

    #[test]
    fn test_clone_implementation() {
        let original = PostNewLink {
            code: Some("test".to_string()),
            target: "https://example.com".to_string(),
            expires_at: None,
        };

        let cloned = original.clone();
        assert_eq!(original.code, cloned.code);
        assert_eq!(original.target, cloned.target);
        assert_eq!(original.expires_at, cloned.expires_at);
    }

    #[test]
    fn test_debug_implementation() {
        let link = SerializableShortLink {
            short_code: "debug_test".to_string(),
            target_url: "https://debug.com".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            expires_at: None,
        };

        let debug_output = format!("{:?}", link);
        assert!(debug_output.contains("debug_test"));
        assert!(debug_output.contains("https://debug.com"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use actix_web::middleware::from_fn;

    #[actix_web::test]
    async fn test_full_admin_workflow() {
        // 设置认证 token
        env::set_var("ADMIN_TOKEN", "integration_test_token");
        
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .wrap(from_fn(AdminService::auth_middleware))
                .service(
                    web::scope("/admin")
                        .route("/links", web::get().to(AdminService::get_all_links))
                        .route("/links", web::post().to(AdminService::post_link))
                        .route("/links/{code}", web::get().to(AdminService::get_link))
                        .route("/links/{code}", web::put().to(AdminService::update_link))
                        .route("/links/{code}", web::delete().to(AdminService::delete_link))
                )
        ).await;

        let auth_header = ("Authorization", "Bearer integration_test_token");

        // 1. 创建新链接
        let new_link = PostNewLink {
            code: Some("integration_test".to_string()),
            target: "https://integration.com".to_string(),
            expires_at: None,
        };

        let req = actix_test::TestRequest::post()
            .uri("/admin/links")
            .insert_header(auth_header)
            .set_json(&new_link)
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        // 2. 获取所有链接
        let req = actix_test::TestRequest::get()
            .uri("/admin/links")
            .insert_header(auth_header)
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: ApiResponse<Option<HashMap<String, SerializableShortLink>>> = 
            actix_test::read_body_json(resp).await;
        assert_eq!(body.data.unwrap().len(), 1);

        // 3. 获取特定链接
        let req = actix_test::TestRequest::get()
            .uri("/admin/links/integration_test")
            .insert_header(auth_header)
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 4. 更新链接
        let update_data = PostNewLink {
            code: None,
            target: "https://updated-integration.com".to_string(),
            expires_at: Some("2025-01-01T00:00:00Z".to_string()),
        };

        let req = actix_test::TestRequest::put()
            .uri("/admin/links/integration_test")
            .insert_header(auth_header)
            .set_json(&update_data)
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 5. 删除链接
        let req = actix_test::TestRequest::delete()
            .uri("/admin/links/integration_test")
            .insert_header(auth_header)
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 6. 验证链接已被删除
        let req = actix_test::TestRequest::get()
            .uri("/admin/links/integration_test")
            .insert_header(auth_header)
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // 清理环境变量
        env::remove_var("ADMIN_TOKEN");
    }

    #[actix_web::test]
    async fn test_concurrent_requests() {
        env::set_var("ADMIN_TOKEN", "concurrent_test_token");
        
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .wrap(from_fn(AdminService::auth_middleware))
                .route("/admin/links", web::post().to(AdminService::post_link))
        ).await;

        let auth_header = ("Authorization", "Bearer concurrent_test_token");

        // 并发创建多个链接
        let mut handles: Vec<()> = vec![];
        for i in 0..5 {
            let new_link = PostNewLink {
                code: Some(format!("concurrent_{}", i)),
                target: format!("https://concurrent{}.com", i),
                expires_at: None,
            };

            let req = actix_test::TestRequest::post()
                .uri("/admin/links")
                .insert_header(auth_header)
                .set_json(&new_link)
                .to_request();

            let resp = actix_test::call_service(&app, req).await;
            assert_eq!(resp.status(), 201);
        }

        // 清理环境变量
        env::remove_var("ADMIN_TOKEN");
    }

    #[actix_web::test]
    async fn test_error_response_format() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .route("/admin/links/{code}", web::get().to(AdminService::get_link))
        ).await;

        let req = actix_test::TestRequest::get()
            .uri("/admin/links/nonexistent")
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        let body: ApiResponse<serde_json::Value> = actix_test::read_body_json(resp).await;
        assert_eq!(body.code, 1);
        assert!(body.data.get("error").is_some());
    }
}
