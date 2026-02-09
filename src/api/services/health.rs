use actix_web::{HttpResponse, Responder, web};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, trace};

use crate::api::services::admin::{
    ApiResponse, ErrorCode, HealthCacheCheck, HealthChecks, HealthResponse, HealthStorageBackend,
    HealthStorageCheck,
};
use crate::cache::CompositeCacheTrait;
use crate::storage::SeaOrmStorage;
use crate::utils::TimeParser;

#[cfg(feature = "metrics")]
use super::metrics::MetricsService;

// 应用启动时间结构体
#[derive(Clone, Debug)]
pub struct AppStartTime {
    pub start_datetime: chrono::DateTime<chrono::Utc>,
}

/// Health Service
///
/// 注意：此 service 直接调用 storage 方法，不通过 LinkService。
/// 这是合理的例外，因为：
/// 1. 基础设施，需要简单直接（k8s probes 要求快速响应）
/// 2. Storage 层方法已足够语义化（count, get_backend_config）
/// 3. 健康检查不应依赖复杂的业务逻辑
pub struct HealthService;

impl HealthService {
    pub async fn health_check(
        storage: web::Data<Arc<SeaOrmStorage>>,
        cache: web::Data<Arc<dyn CompositeCacheTrait>>,
        app_start_time: web::Data<AppStartTime>,
    ) -> impl Responder {
        let start_time = Instant::now();
        trace!("Received health check request");

        // 获取后端配置
        let backend_config = storage.get_backend_config().await;
        let backend = HealthStorageBackend {
            storage_type: backend_config.storage_type,
            support_click: backend_config.support_click,
        };

        // 检查存储健康状况（只查 count，不加载全表）
        let storage_status =
            match tokio::time::timeout(Duration::from_secs(5), storage.count()).await {
                Ok(Ok(count)) => {
                    trace!("Storage health check passed, {} links found", count);
                    HealthStorageCheck {
                        status: "healthy".to_string(),
                        links_count: Some(count as usize),
                        backend,
                        error: None,
                    }
                }
                Ok(Err(e)) => {
                    error!("Storage health check failed: {}", e);
                    HealthStorageCheck {
                        status: "unhealthy".to_string(),
                        links_count: None,
                        backend,
                        error: Some(format!("database error: {}", e)),
                    }
                }
                Err(_) => {
                    error!("Storage health check timeout");
                    HealthStorageCheck {
                        status: "unhealthy".to_string(),
                        links_count: None,
                        backend,
                        error: Some("timeout".to_string()),
                    }
                }
            };

        // 检查缓存健康状况
        let cache_health = cache.health_check().await;
        let cache_status = Some(HealthCacheCheck {
            status: cache_health.status,
            cache_type: cache_health.cache_type,
            bloom_filter_enabled: cache_health.bloom_filter_enabled,
            negative_cache_enabled: cache_health.negative_cache_enabled,
            error: cache_health.error,
        });

        let now = chrono::Utc::now();

        // 使用 TimeParser 的方法格式化运行时间
        let uptime_human = TimeParser::format_duration_human(app_start_time.start_datetime, now);

        // 计算运行秒数
        let uptime_seconds = (now - app_start_time.start_datetime).num_seconds().max(0) as u32;

        let is_healthy = storage_status.status == "healthy";

        let health_data = HealthResponse {
            status: if is_healthy {
                "healthy".to_string()
            } else {
                "unhealthy".to_string()
            },
            timestamp: now.to_rfc3339(),
            uptime: uptime_seconds,
            checks: HealthChecks {
                storage: storage_status,
                cache: cache_status,
            },
            response_time_ms: start_time.elapsed().as_millis() as u32,
        };

        let health_response = ApiResponse {
            code: if is_healthy {
                ErrorCode::Success as i32
            } else {
                ErrorCode::ServiceUnavailable as i32
            },
            message: if is_healthy {
                "OK".to_string()
            } else {
                "Service Unavailable".to_string()
            },
            data: Some(health_data),
        };

        let response_status = if is_healthy {
            actix_web::http::StatusCode::OK
        } else {
            actix_web::http::StatusCode::SERVICE_UNAVAILABLE
        };

        info!(
            "Health check completed in {:?}, status: {}, uptime: {}",
            start_time.elapsed(),
            if is_healthy { "healthy" } else { "unhealthy" },
            uptime_human
        );

        HttpResponse::build(response_status)
            .append_header(("Content-Type", "application/json; charset=utf-8"))
            .json(health_response)
    }

    // 简单的就绪检查，只返回 200 状态码
    pub async fn readiness_check() -> impl Responder {
        trace!("Received readiness check request");

        HttpResponse::Ok()
            .append_header(("Content-Type", "text/plain"))
            .body("OK")
    }

    // 活跃性检查，检查基本服务可用性
    pub async fn liveness_check() -> impl Responder {
        trace!("Received liveness check request");

        HttpResponse::NoContent().finish()
    }
}

/// Health 路由配置
pub fn health_routes() -> actix_web::Scope {
    let scope = web::scope("")
        .route("", web::get().to(HealthService::health_check))
        .route("", web::head().to(HealthService::health_check))
        .route("/ready", web::get().to(HealthService::readiness_check))
        .route("/ready", web::head().to(HealthService::readiness_check))
        .route("/live", web::get().to(HealthService::liveness_check))
        .route("/live", web::head().to(HealthService::liveness_check));

    #[cfg(feature = "metrics")]
    let scope = scope.route("/metrics", web::get().to(MetricsService::metrics));

    scope
}
