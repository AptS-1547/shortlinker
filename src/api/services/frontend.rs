use actix_web::{HttpRequest, HttpResponse, Result};
use rust_embed::Embed;
use std::env;
use std::sync::OnceLock;
use tracing::{debug, trace};

// 使用 RustEmbed 自动嵌入静态文件
#[derive(Embed)]
#[folder = "admin-panel/dist/"]
struct FrontendAssets;

pub struct FrontendService;

static FRONTEND_ROUTE_PREFIX: OnceLock<String> = OnceLock::new();
static ADMIN_ROUTE_PREFIX: OnceLock<String> = OnceLock::new();
static HEALTH_ROUTE_PREFIX: OnceLock<String> = OnceLock::new();

impl FrontendService {
    /// 处理前端首页 - 服务构建好的 index.html
    pub async fn handle_index(req: HttpRequest) -> Result<HttpResponse> {
        trace!("Serving frontend index page from dist");

        let config = crate::config::get_config();
        let frontend_prefix =
            FRONTEND_ROUTE_PREFIX.get_or_init(|| config.routes.frontend_prefix.clone());
        let admin_prefix = ADMIN_ROUTE_PREFIX.get_or_init(|| config.routes.admin_prefix.clone());
        let health_prefix = HEALTH_ROUTE_PREFIX.get_or_init(|| config.routes.health_prefix.clone());

        // 检查路径是否需要规范化（添加尾部斜杠）
        let path = req.path();
        if path == frontend_prefix && !path.ends_with('/') {
            trace!("Redirecting {} to {}/", path, path);
            return Ok(HttpResponse::Found()
                .append_header(("Location", format!("{}/", path)))
                .finish());
        }

        match FrontendAssets::get("index.html") {
            Some(content) => {
                // 将字节数组转换为字符串并替换占位符
                let html_content = String::from_utf8_lossy(&content.data);
                let processed_html = html_content
                    .replace("%BASE_PATH%", frontend_prefix)
                    .replace("%ADMIN_ROUTE_PREFIX%", admin_prefix)
                    .replace("%HEALTH_ROUTE_PREFIX%", health_prefix)
                    .replace("%SHORTLINKER_VERSION%", env!("CARGO_PKG_VERSION"));

                Ok(HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(processed_html))
            }
            None => {
                // 使用编译时包含作为后备
                let fallback_html = include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/admin-panel/dist/index.html"
                ));
                let processed_html = fallback_html
                    .replace("%BASE_PATH%", frontend_prefix)
                    .replace("%ADMIN_ROUTE_PREFIX%", admin_prefix)
                    .replace("%HEALTH_ROUTE_PREFIX%", health_prefix)
                    .replace("%SHORTLINKER_VERSION%", env!("CARGO_PKG_VERSION"));
                Ok(HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(processed_html))
            }
        }
    }

    /// 处理静态资源文件
    pub async fn handle_static(req: HttpRequest) -> Result<HttpResponse> {
        let path = req.match_info().query("path");
        trace!("Serving static file: {}", path);

        // 根据文件扩展名确定 Content-Type
        let content_type = Self::get_content_type(path);

        let asset_path = format!("assets/{}", path);

        match FrontendAssets::get(&asset_path) {
            Some(content) => Ok(HttpResponse::Ok()
                .content_type(content_type)
                .body(content.data.into_owned())),
            None => {
                debug!("Static file not found: {}", path);
                Ok(HttpResponse::NotFound().body("File not found"))
            }
        }
    }

    /// 处理 favicon.ico 请求
    pub async fn handle_favicon(_req: HttpRequest) -> Result<HttpResponse> {
        trace!("Serving favicon");

        match FrontendAssets::get("favicon.ico") {
            Some(favicon_data) => Ok(HttpResponse::Ok()
                .content_type("image/x-icon")
                .body(favicon_data.data.into_owned())),
            None => {
                // 如果没有找到，返回空的 favicon
                Ok(HttpResponse::Ok().content_type("image/x-icon").body(vec![]))
            }
        }
    }

    /// 处理管理面板页面 - 重定向到根路径或直接服务
    pub async fn handle_admin_panel(_req: HttpRequest) -> Result<HttpResponse> {
        trace!("Redirecting to admin panel");

        // 如果访问 /admin，重定向到根路径（前端路由会处理）
        Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish())
    }

    /// 处理所有未匹配的路由，返回 index.html（用于 SPA 路由）
    pub async fn handle_spa_fallback(req: HttpRequest) -> Result<HttpResponse> {
        trace!("SPA fallback - serving index.html");

        let config = crate::config::get_config();
        let frontend_prefix =
            FRONTEND_ROUTE_PREFIX.get_or_init(|| config.routes.frontend_prefix.clone());
        let admin_prefix = ADMIN_ROUTE_PREFIX.get_or_init(|| config.routes.admin_prefix.clone());
        let health_prefix = HEALTH_ROUTE_PREFIX.get_or_init(|| config.routes.health_prefix.clone());

        // 检查路径是否需要规范化（添加尾部斜杠）
        let path = req.path();
        if path == frontend_prefix && !path.ends_with('/') {
            trace!("Redirecting {} to {}/", path, path);
            return Ok(HttpResponse::Found()
                .append_header(("Location", format!("{}/", path)))
                .finish());
        }

        match FrontendAssets::get("index.html") {
            Some(content) => {
                // 将字节数组转换为字符串并替换占位符
                let html_content = String::from_utf8_lossy(&content.data);
                let processed_html = html_content
                    .replace("%BASE_PATH%", frontend_prefix)
                    .replace("%ADMIN_ROUTE_PREFIX%", admin_prefix)
                    .replace("%HEALTH_ROUTE_PREFIX%", health_prefix)
                    .replace("%SHORTLINKER_VERSION%", env!("CARGO_PKG_VERSION"));

                Ok(HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(processed_html))
            }
            None => {
                // 如果没有找到，使用直接包含的方式作为后备
                let html = include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/admin-panel/dist/index.html"
                ));
                let processed_html = html.replace("%BASE_PATH%", frontend_prefix);
                Ok(HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(processed_html))
            }
        }
    }

    /// 根据文件扩展名确定 Content-Type
    fn get_content_type(path: &str) -> &'static str {
        match path.split('.').next_back() {
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
        }
    }
}
