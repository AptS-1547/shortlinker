//! Health check handler - 基础设施路径直连策略
//!
//! # 架构决策：直连 Storage + Cache
//!
//! health handler 直接访问 `SeaOrmStorage` 和产品侧 `LinkCache` policy，
//! 不经过任何 service 层。这是有意为之的设计决策：
//!
//! ## 原因
//! 1. **独立性**：health check 用于监控系统健康状态，
//!    不应依赖业务 service 层（如果 service 层有 bug，health check 不应受影响）
//! 2. **直接性**：health check 需要直接探测底层组件（DB 连接、缓存状态），
//!    service 层的抽象反而会掩盖真实的健康状态
//! 3. **k8s 探针**：readiness/liveness 探针需要最小依赖链

use actix_web::{HttpResponse, Responder, web};
use aster_forge_runtime::{
    HealthCheckOptions, HealthCheckRegistry, HealthCheckScope, HealthCheckScopes,
    HealthComponentReport, HealthStatus,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, trace};

use crate::api::services::admin::{
    ApiResponse, ErrorCode, HealthCacheCheck, HealthChecks, HealthResponse, HealthStorageBackend,
    HealthStorageCheck,
};
use crate::services::LinkCache;
use crate::storage::SeaOrmStorage;
use crate::utils::TimeParser;

// 应用启动时间结构体
#[derive(Clone, Debug)]
pub struct AppStartTime {
    pub start_datetime: chrono::DateTime<chrono::Utc>,
}

pub struct HealthService;

fn health_registry(storage: Arc<SeaOrmStorage>, cache: Arc<dyn LinkCache>) -> HealthCheckRegistry {
    HealthCheckRegistry::configured(|registry| {
        let storage = storage.clone();
        registry.register_with_options(
            "storage",
            HealthCheckOptions::required(Some(Duration::from_secs(5)))
                .with_scopes(HealthCheckScopes::readiness_and_diagnostics()),
            move || {
                let storage = storage.clone();
                async move {
                    match storage.count().await {
                        Ok(count) => HealthComponentReport::healthy(
                            "storage",
                            format!("database available with {count} links"),
                        )
                        .with_detail("links_count", count),
                        Err(error) => {
                            error!(%error, "storage health check failed");
                            HealthComponentReport::unhealthy(
                                "storage",
                                format!("database error: {error}"),
                            )
                        }
                    }
                }
            },
        );

        registry.register_with_options(
            "cache",
            HealthCheckOptions::optional(Some(Duration::from_secs(5)))
                .with_scopes(HealthCheckScopes::readiness_and_diagnostics()),
            move || {
                let cache = cache.clone();
                async move {
                    let health = cache.health_check().await;
                    let mut report = if health.status == "healthy" {
                        HealthComponentReport::healthy("cache", "cache available")
                    } else {
                        HealthComponentReport::degraded(
                            "cache",
                            health
                                .error
                                .clone()
                                .unwrap_or_else(|| "cache unavailable".to_string()),
                        )
                    }
                    .with_detail("reported_status", health.status)
                    .with_detail("cache_type", health.cache_type)
                    .with_detail("bloom_filter_enabled", health.bloom_filter_enabled)
                    .with_detail("negative_cache_enabled", health.negative_cache_enabled);

                    if let Some(error) = health.error {
                        report = report.with_detail("error", error);
                    }
                    report
                }
            },
        );
    })
}

impl HealthService {
    pub async fn health_check(
        storage: web::Data<Arc<SeaOrmStorage>>,
        cache: web::Data<Arc<dyn LinkCache>>,
        app_start_time: web::Data<AppStartTime>,
    ) -> impl Responder {
        trace!("Received health check request");

        let registry = health_registry(storage.get_ref().clone(), cache.get_ref().clone());
        let report = registry.run_scope(HealthCheckScope::Diagnostics).await;

        let backend_config = storage.get_backend_config().await;
        let backend = HealthStorageBackend {
            storage_type: backend_config.storage_type,
            support_click: backend_config.support_click,
        };

        let storage_status = report
            .components
            .iter()
            .find(|component| component.name == "storage")
            .map(|component| HealthStorageCheck {
                status: component.status.as_str().to_string(),
                links_count: component
                    .detail("links_count")
                    .and_then(|value| value.as_unsigned())
                    .and_then(|count| usize::try_from(count).ok()),
                backend: backend.clone(),
                error: component
                    .status
                    .is_issue()
                    .then(|| component.message.clone()),
            })
            .unwrap_or_else(|| HealthStorageCheck {
                status: "unhealthy".to_string(),
                links_count: None,
                backend,
                error: Some("storage health check missing".to_string()),
            });

        let cache_status = report
            .components
            .iter()
            .find(|component| component.name == "cache")
            .map(|component| HealthCacheCheck {
                status: component
                    .detail("reported_status")
                    .and_then(|value| value.as_text())
                    .unwrap_or_else(|| component.status.as_str())
                    .to_string(),
                cache_type: component
                    .detail("cache_type")
                    .and_then(|value| value.as_text())
                    .unwrap_or("unknown")
                    .to_string(),
                bloom_filter_enabled: component
                    .detail("bloom_filter_enabled")
                    .and_then(|value| value.as_boolean())
                    .unwrap_or(false),
                negative_cache_enabled: component
                    .detail("negative_cache_enabled")
                    .and_then(|value| value.as_boolean())
                    .unwrap_or(false),
                error: component
                    .detail("error")
                    .and_then(|value| value.as_text())
                    .map(str::to_string)
                    .or_else(|| {
                        component
                            .status
                            .is_issue()
                            .then(|| component.message.clone())
                    }),
            });

        let now = chrono::Utc::now();

        // 使用 TimeParser 的方法格式化运行时间
        let uptime_human = TimeParser::format_duration_human(app_start_time.start_datetime, now);

        // 计算运行秒数
        let uptime_seconds =
            u32::try_from((now - app_start_time.start_datetime).num_seconds().max(0))
                .unwrap_or(u32::MAX);
        let status = report.status();

        let health_data = HealthResponse {
            status: status.as_str().to_string(),
            timestamp: now.to_rfc3339(),
            uptime: uptime_seconds,
            checks: HealthChecks {
                storage: storage_status,
                cache: cache_status,
            },
            response_time_ms: report
                .duration
                .and_then(|duration| u32::try_from(duration.as_millis()).ok())
                .unwrap_or(u32::MAX),
        };

        let health_response = ApiResponse {
            code: if matches!(status, HealthStatus::Unhealthy) {
                ErrorCode::ServiceUnavailable as i32
            } else {
                ErrorCode::Success as i32
            },
            message: if matches!(status, HealthStatus::Unhealthy) {
                "Service Unavailable".to_string()
            } else {
                "OK".to_string()
            },
            data: Some(health_data),
        };

        let response_status = if matches!(status, HealthStatus::Unhealthy) {
            actix_web::http::StatusCode::SERVICE_UNAVAILABLE
        } else {
            actix_web::http::StatusCode::OK
        };

        info!(
            "Health check completed in {:?}, status: {}, uptime: {}",
            report.duration.unwrap_or_default(),
            status.as_str(),
            uptime_human
        );

        HttpResponse::build(response_status)
            .append_header(("Content-Type", "application/json; charset=utf-8"))
            .json(health_response)
    }

    pub async fn readiness_check(
        storage: web::Data<Arc<SeaOrmStorage>>,
        cache: web::Data<Arc<dyn LinkCache>>,
    ) -> impl Responder {
        trace!("Received readiness check request");

        let registry = health_registry(storage.get_ref().clone(), cache.get_ref().clone());
        let report = registry.run_scope(HealthCheckScope::Readiness).await;
        if matches!(report.status(), HealthStatus::Unhealthy) {
            HttpResponse::ServiceUnavailable()
                .append_header(("Content-Type", "text/plain"))
                .body("Service Unavailable")
        } else {
            HttpResponse::Ok()
                .append_header(("Content-Type", "text/plain"))
                .body("OK")
        }
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

    aster_forge_actix_observability::configure_prometheus_route(scope)
}
