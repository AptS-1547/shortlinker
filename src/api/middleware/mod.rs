/// 管理接口的认证（Bearer Token/JWT/Cookie）中间件
pub mod auth;
/// 基于 Cookie 的 CSRF 防护中间件
pub mod csrf;
/// 管理面板静态资源的守卫中间件
pub mod frontend;
/// 健康检查端点的认证中间件
pub mod health;

pub use auth::{AdminAuth, AuthMethod};
pub use csrf::CsrfGuard;
pub use frontend::FrontendGuard;
pub use health::HealthAuth;
