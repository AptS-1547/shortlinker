use actix_web::{HttpResponse, Responder, web};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info};

use crate::storages::Storage;
use crate::utils::TimeParser;

// 应用启动时间结构体
#[derive(Clone, Debug)]
pub struct AppStartTime {
    pub start_datetime: chrono::DateTime<chrono::Utc>,
}

pub struct HealthService;

impl HealthService {
    pub async fn health_check(
        storage: web::Data<Arc<dyn Storage>>,
        app_start_time: web::Data<AppStartTime>,
    ) -> impl Responder {
        let start_time = Instant::now();
        debug!("Received health check request");

        // 检查存储健康状况
        let storage_status =
            match tokio::time::timeout(Duration::from_secs(5), storage.load_all()).await {
                Ok(links) => {
                    debug!("Storage health check passed, {} links found", links.len());
                    json!({
                        "status": "healthy",
                        "links_count": links.len(),
                        "backend": storage.get_backend_config().await
                    })
                }
                Err(_) => {
                    error!("Storage health check timeout");
                    json!({
                        "status": "unhealthy",
                        "error": "timeout",
                        "backend": storage.get_backend_config().await
                    })
                }
            };

        let now = chrono::Utc::now();

        // 使用 TimeParser 的方法格式化运行时间
        let uptime_human = TimeParser::format_duration_human(app_start_time.start_datetime, now);

        // 计算运行秒数
        let uptime_seconds = (now - app_start_time.start_datetime).num_seconds().max(0) as u64;

        let is_healthy = storage_status["status"] == "healthy";

        let health_response = json!({
            "status": if is_healthy { "healthy" } else { "unhealthy" },
            "timestamp": now.to_rfc3339(),
            "uptime": uptime_seconds,
            "checks": {
                "storage": storage_status,
            },
            "response_time_ms": start_time.elapsed().as_millis()
        });

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
        debug!("Received readiness check request");

        HttpResponse::Ok()
            .append_header(("Content-Type", "text/plain"))
            .body("OK")
    }

    // 活跃性检查，检查基本服务可用性
    pub async fn liveness_check() -> impl Responder {
        debug!("Received liveness check request");

        HttpResponse::NoContent().finish()
    }
}
