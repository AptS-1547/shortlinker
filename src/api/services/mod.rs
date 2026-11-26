/// 管理 API 的 handler 与路由
pub mod admin;
/// 管理面板静态资源与 SPA fallback 服务
pub mod frontend;
/// 健康/就绪/存活检查端点
pub mod health;
/// 短链接重定向热路径
pub mod redirect;

pub use frontend::{FrontendService, frontend_routes};
pub use health::{AppStartTime, HealthService, health_routes};
pub use redirect::{RedirectService, redirect_routes};
