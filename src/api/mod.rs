/// HTTP 层使用的常量定义
pub mod constants;
/// 管理 API 的 JWT 签发与校验
pub mod jwt;
/// 认证、CSRF、前端守卫等 Actix 中间件
pub mod middleware;
#[cfg(all(debug_assertions, feature = "openapi"))]
pub mod openapi;
/// 管理 API、健康检查、重定向和前端静态资源的 handler
pub mod services;
