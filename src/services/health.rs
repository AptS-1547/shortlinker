use actix_web::{web, HttpResponse, Responder};
use log::{debug, error, info};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::storages::Storage;

// 应用启动时间结构体
#[derive(Clone, Debug)]
pub struct AppStartTime {
    pub start_time: Instant,
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
                        "backend": storage.get_backend_name().await
                    })
                }
                Err(_) => {
                    error!("Storage health check timeout");
                    json!({
                        "status": "unhealthy",
                        "error": "timeout",
                        "backend": storage.get_backend_name().await
                    })
                }
            };

        // 计算程序运行时间
        let uptime_duration = app_start_time.start_time.elapsed();
        let uptime_seconds = uptime_duration.as_secs();

        let is_healthy = storage_status["status"] == "healthy";

        let health_response = json!({
            "status": if is_healthy { "healthy" } else { "unhealthy" },
            "timestamp": chrono::Utc::now().to_rfc3339(),
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
            Self::format_duration(uptime_duration)
        );

        HttpResponse::build(response_status)
            .append_header(("Content-Type", "application/json; charset=utf-8"))
            .append_header(("Connection", "close"))
            .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
            .json(health_response)
    }

    // 简单的就绪检查，只返回 200 状态码
    pub async fn readiness_check() -> impl Responder {
        debug!("Received readiness check request");

        HttpResponse::Ok()
            .append_header(("Content-Type", "text/plain"))
            .append_header(("Connection", "close"))
            .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
            .body("OK")
    }

    // 活跃性检查，检查基本服务可用性
    pub async fn liveness_check() -> impl Responder {
        debug!("Received liveness check request");

        HttpResponse::NoContent()
            .append_header(("Connection", "close"))
            .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
            .finish()
    }

    // 格式化运行时间为人类可读格式
    fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}
