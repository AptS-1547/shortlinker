//! Admin API 服务模块
//!
//! 该模块包含管理 API 的所有端点，包括：
//! - 认证（登录、登出、token 刷新）
//! - 链接 CRUD 操作
//! - 批量操作
//! - 配置管理

pub mod auth;
mod batch_ops;
mod config_ops;
pub mod error_code;
mod export_import;
mod helpers;
mod link_crud;
pub mod routes;
mod types;

// 重新导出类型
pub use types::*;

// 重新导出帮助函数
pub use helpers::{
    api_result, error_from_shortlinker, error_response, parse_expires_at, success_response,
};

// 重新导出错误码
pub use error_code::ErrorCode;

// 重新导出认证端点
pub use auth::{check_admin_token, logout, refresh_token, verify_token};

// 重新导出链接 CRUD 端点
pub use link_crud::{delete_link, get_all_links, get_link, get_stats, post_link, update_link};

// 重新导出批量操作端点
pub use batch_ops::{batch_create_links, batch_delete_links, batch_update_links};

// 重新导出导出导入端点
pub use export_import::{export_links, import_links};

// 重新导出配置管理端点
pub use config_ops::{
    ConfigHistoryResponse, ConfigItemResponse, ConfigUpdateRequest, ConfigUpdateResponse,
    get_all_configs, get_config, get_config_history, get_config_schema, reload_config,
    update_config,
};

/// 获取随机码长度配置
pub(crate) fn get_random_code_length() -> usize {
    crate::config::get_config().features.random_code_length
}

/// AdminService 结构体，保持向后兼容
pub struct AdminService;

impl AdminService {
    pub async fn get_all_links(
        req: actix_web::HttpRequest,
        query: actix_web::web::Query<GetLinksQuery>,
        service: actix_web::web::Data<std::sync::Arc<crate::services::LinkService>>,
    ) -> actix_web::Result<impl actix_web::Responder> {
        link_crud::get_all_links(req, query, service).await
    }

    pub async fn post_link(
        req: actix_web::HttpRequest,
        link: actix_web::web::Json<PostNewLink>,
        service: actix_web::web::Data<std::sync::Arc<crate::services::LinkService>>,
    ) -> actix_web::Result<impl actix_web::Responder> {
        link_crud::post_link(req, link, service).await
    }

    pub async fn get_link(
        req: actix_web::HttpRequest,
        code: actix_web::web::Path<String>,
        service: actix_web::web::Data<std::sync::Arc<crate::services::LinkService>>,
    ) -> actix_web::Result<impl actix_web::Responder> {
        link_crud::get_link(req, code, service).await
    }

    pub async fn delete_link(
        req: actix_web::HttpRequest,
        code: actix_web::web::Path<String>,
        service: actix_web::web::Data<std::sync::Arc<crate::services::LinkService>>,
    ) -> actix_web::Result<impl actix_web::Responder> {
        link_crud::delete_link(req, code, service).await
    }

    pub async fn update_link(
        req: actix_web::HttpRequest,
        code: actix_web::web::Path<String>,
        link: actix_web::web::Json<PostNewLink>,
        service: actix_web::web::Data<std::sync::Arc<crate::services::LinkService>>,
    ) -> actix_web::Result<impl actix_web::Responder> {
        link_crud::update_link(req, code, link, service).await
    }

    pub async fn check_admin_token(
        req: actix_web::HttpRequest,
        login_body: actix_web::web::Json<LoginCredentials>,
    ) -> actix_web::Result<impl actix_web::Responder> {
        auth::check_admin_token(req, login_body).await
    }

    pub async fn refresh_token(
        req: actix_web::HttpRequest,
    ) -> actix_web::Result<impl actix_web::Responder> {
        auth::refresh_token(req).await
    }

    pub async fn logout(
        req: actix_web::HttpRequest,
    ) -> actix_web::Result<impl actix_web::Responder> {
        auth::logout(req).await
    }

    pub async fn verify_token(
        req: actix_web::HttpRequest,
    ) -> actix_web::Result<impl actix_web::Responder> {
        auth::verify_token(req).await
    }

    pub async fn batch_create_links(
        req: actix_web::HttpRequest,
        batch: actix_web::web::Json<BatchCreateRequest>,
        cache: actix_web::web::Data<std::sync::Arc<dyn crate::cache::traits::CompositeCacheTrait>>,
        storage: actix_web::web::Data<std::sync::Arc<crate::storage::SeaOrmStorage>>,
    ) -> actix_web::Result<impl actix_web::Responder> {
        batch_ops::batch_create_links(req, batch, cache, storage).await
    }

    pub async fn batch_update_links(
        req: actix_web::HttpRequest,
        batch: actix_web::web::Json<BatchUpdateRequest>,
        cache: actix_web::web::Data<std::sync::Arc<dyn crate::cache::traits::CompositeCacheTrait>>,
        storage: actix_web::web::Data<std::sync::Arc<crate::storage::SeaOrmStorage>>,
    ) -> actix_web::Result<impl actix_web::Responder> {
        batch_ops::batch_update_links(req, batch, cache, storage).await
    }

    pub async fn batch_delete_links(
        req: actix_web::HttpRequest,
        batch: actix_web::web::Json<BatchDeleteRequest>,
        cache: actix_web::web::Data<std::sync::Arc<dyn crate::cache::traits::CompositeCacheTrait>>,
        storage: actix_web::web::Data<std::sync::Arc<crate::storage::SeaOrmStorage>>,
    ) -> actix_web::Result<impl actix_web::Responder> {
        batch_ops::batch_delete_links(req, batch, cache, storage).await
    }

    pub async fn get_stats(
        req: actix_web::HttpRequest,
        service: actix_web::web::Data<std::sync::Arc<crate::services::LinkService>>,
    ) -> actix_web::Result<impl actix_web::Responder> {
        link_crud::get_stats(req, service).await
    }

    pub async fn export_links(
        req: actix_web::HttpRequest,
        query: actix_web::web::Query<ExportQuery>,
        storage: actix_web::web::Data<std::sync::Arc<crate::storage::SeaOrmStorage>>,
    ) -> actix_web::Result<impl actix_web::Responder> {
        export_import::export_links(req, query, storage).await
    }

    pub async fn import_links(
        req: actix_web::HttpRequest,
        payload: actix_multipart::Multipart,
        cache: actix_web::web::Data<std::sync::Arc<dyn crate::cache::traits::CompositeCacheTrait>>,
        storage: actix_web::web::Data<std::sync::Arc<crate::storage::SeaOrmStorage>>,
    ) -> actix_web::Result<impl actix_web::Responder> {
        export_import::import_links(req, payload, cache, storage).await
    }
}
