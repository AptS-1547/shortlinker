use actix_web::{HttpRequest, HttpResponse, Result};
use std::collections::HashMap;
use std::env;
use std::sync::OnceLock;
use tracing::debug;

pub struct FrontendService;

// 静态文件映射表，在编译时生成
static STATIC_FILES: OnceLock<HashMap<String, &'static [u8]>> = OnceLock::new();

static FRONTEND_ROUTE_PREFIX: OnceLock<String> = OnceLock::new();
static ADMIN_ROUTE_PREFIX: OnceLock<String> = OnceLock::new();
static HEALTH_ROUTE_PREFIX: OnceLock<String> = OnceLock::new();

impl FrontendService {
    /// 初始化静态文件映射
    fn init_static_files() -> HashMap<String, &'static [u8]> {
        let mut files: HashMap<String, &'static [u8]> = HashMap::new();

        // HTML 文件
        files.insert(
            "index.html".to_string(),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/admin-panel/dist/index.html"
            )),
        );

        // Favicon
        files.insert(
            "favicon.ico".to_string(),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/admin-panel/dist/favicon.ico"
            )),
        );

        // 动态包含 assets 目录下的所有文件
        // 这部分会通过 build.rs 在编译时生成
        include!(concat!(env!("OUT_DIR"), "/static_files.rs"));

        files
    }

    /// 获取静态文件映射
    fn get_static_files() -> &'static HashMap<String, &'static [u8]> {
        STATIC_FILES.get_or_init(Self::init_static_files)
    }

    /// 处理前端首页 - 服务构建好的 index.html
    pub async fn handle_index(req: HttpRequest) -> Result<HttpResponse> {
        debug!("Serving frontend index page from dist");

        let frontend_prefix = FRONTEND_ROUTE_PREFIX.get_or_init(|| {
            env::var("FRONTEND_ROUTE_PREFIX").unwrap_or_else(|_| "/panel".to_string())
        });
        let admin_prefix = ADMIN_ROUTE_PREFIX.get_or_init(|| {
            env::var("ADMIN_ROUTE_PREFIX").unwrap_or_else(|_| "/admin".to_string())
        });
        let health_prefix = HEALTH_ROUTE_PREFIX.get_or_init(|| {
            env::var("HEALTH_ROUTE_PREFIX").unwrap_or_else(|_| "/health".to_string())
        });

        // 检查路径是否需要规范化（添加尾部斜杠）
        let path = req.path();
        if path == frontend_prefix && !path.ends_with('/') {
            debug!("Redirecting {} to {}/", path, path);
            return Ok(HttpResponse::Found()
                .append_header(("Location", format!("{}/", path)))
                .finish());
        }

        let files = Self::get_static_files();
        match files.get("index.html") {
            Some(content) => {
                // 将字节数组转换为字符串并替换占位符
                let html_content = String::from_utf8_lossy(content);
                let processed_html = html_content
                    .replace("%BASE_PATH%", &frontend_prefix)
                    .replace("%ADMIN_ROUTE_PREFIX%", &admin_prefix)
                    .replace("%HEALTH_ROUTE_PREFIX%", &health_prefix);

                Ok(HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(processed_html))
            }
            None => {
                let fallback_html = include_str!("../../admin-panel/dist/index.html");
                let processed_html = fallback_html
                    .replace("%BASE_PATH%", &frontend_prefix)
                    .replace("%ADMIN_ROUTE_PREFIX%", &admin_prefix)
                    .replace("%HEALTH_ROUTE_PREFIX%", &health_prefix);
                Ok(HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(processed_html))
            }
        }
    }

    /// 处理静态资源文件
    pub async fn handle_static(req: HttpRequest) -> Result<HttpResponse> {
        let path = req.match_info().query("path");
        debug!("Serving static file: {}", path);

        // 根据文件扩展名确定 Content-Type
        let content_type = match path.split('.').last() {
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("woff") => "font/woff",
            Some("woff2") => "font/woff2",
            Some("ttf") => "font/ttf",
            Some("eot") => "application/vnd.ms-fontobject",
            _ => "application/octet-stream",
        };

        let files = Self::get_static_files();
        let asset_key = format!("assets/{}", path);

        match files.get(&asset_key) {
            Some(content) => Ok(HttpResponse::Ok()
                .content_type(content_type)
                .body(content.to_vec())),
            None => {
                debug!("Static file not found: {}", path);
                Ok(HttpResponse::NotFound().body("File not found"))
            }
        }
    }

    /// 处理 favicon.ico 请求
    pub async fn handle_favicon(_req: HttpRequest) -> Result<HttpResponse> {
        debug!("Serving favicon");

        let files = Self::get_static_files();
        match files.get("favicon.ico") {
            Some(favicon_data) => Ok(HttpResponse::Ok()
                .content_type("image/x-icon")
                .body(favicon_data.to_vec())),
            None => {
                // 如果没有找到，返回空的 favicon
                Ok(HttpResponse::Ok().content_type("image/x-icon").body(vec![]))
            }
        }
    }

    /// 处理管理面板页面 - 重定向到根路径或直接服务
    pub async fn handle_admin_panel(_req: HttpRequest) -> Result<HttpResponse> {
        debug!("Redirecting to admin panel");

        // 如果访问 /admin，重定向到根路径（前端路由会处理）
        Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish())
    }

    /// 处理所有未匹配的路由，返回 index.html（用于 SPA 路由）
    pub async fn handle_spa_fallback(req: HttpRequest) -> Result<HttpResponse> {
        debug!("SPA fallback - serving index.html");

        let frontend_prefix = FRONTEND_ROUTE_PREFIX.get_or_init(|| {
            env::var("FRONTEND_ROUTE_PREFIX").unwrap_or_else(|_| "/panel".to_string())
        });
        let admin_prefix = ADMIN_ROUTE_PREFIX.get_or_init(|| {
            env::var("ADMIN_ROUTE_PREFIX").unwrap_or_else(|_| "/admin".to_string())
        });
        let health_prefix = HEALTH_ROUTE_PREFIX.get_or_init(|| {
            env::var("HEALTH_ROUTE_PREFIX").unwrap_or_else(|_| "/health".to_string())
        });

        // 检查路径是否需要规范化（添加尾部斜杠）
        let path = req.path();
        if path == frontend_prefix && !path.ends_with('/') {
            debug!("Redirecting {} to {}/", path, path);
            return Ok(HttpResponse::Found()
                .append_header(("Location", format!("{}/", path)))
                .finish());
        }

        let files = Self::get_static_files();
        match files.get("index.html") {
            Some(content) => {
                // 将字节数组转换为字符串并替换占位符
                let html_content = String::from_utf8_lossy(content);
                let processed_html = html_content
                    .replace("%BASE_PATH%", &frontend_prefix)
                    .replace("%ADMIN_ROUTE_PREFIX%", &admin_prefix)
                    .replace("%HEALTH_ROUTE_PREFIX%", &health_prefix);

                Ok(HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(processed_html))
            }
            None => {
                // 如果映射中没有找到，使用直接包含的方式作为后备
                let html = include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/admin-panel/dist/index.html"
                ));
                let processed_html = html.replace("%BASE_PATH%", &frontend_prefix);
                Ok(HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(processed_html))
            }
        }
    }
}
