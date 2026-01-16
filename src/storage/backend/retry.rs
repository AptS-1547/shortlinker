//! 数据库操作重试模块
//!
//! 提供类似 Redis ConnectionManager 的断线重连能力

use sea_orm::DbErr;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// 判断数据库错误是否可重试
pub fn is_retryable_error(err: &DbErr) -> bool {
    matches!(
        err,
        DbErr::ConnectionAcquire(_) | // 连接池获取失败
        DbErr::Conn(_) // 连接问题
    )
}

/// 重试配置
#[derive(Clone, Copy)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 2000,
        }
    }
}

/// 指数退避重试执行器
///
/// 对可重试错误自动进行重试，使用指数退避 + 随机抖动避免惊群效应
pub async fn with_retry<T, F, Fut>(
    operation_name: &str,
    config: RetryConfig,
    mut operation: F,
) -> Result<T, DbErr>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, DbErr>>,
{
    let mut attempt = 0;
    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("操作 '{}' 在第 {} 次重试后成功", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(e) if is_retryable_error(&e) && attempt < config.max_retries => {
                attempt += 1;
                let delay = calculate_backoff(attempt, config.base_delay_ms, config.max_delay_ms);
                warn!(
                    "操作 '{}' 失败 (尝试 {}/{}): {}，{} 毫秒后重试",
                    operation_name,
                    attempt,
                    config.max_retries + 1,
                    e,
                    delay
                );
                sleep(Duration::from_millis(delay)).await;
            }
            Err(e) => {
                if !is_retryable_error(&e) {
                    debug!("操作 '{}' 失败，错误不可重试: {}", operation_name, e);
                }
                return Err(e);
            }
        }
    }
}

/// 计算指数退避延迟（带抖动）
fn calculate_backoff(attempt: u32, base_ms: u64, max_ms: u64) -> u64 {
    use rand::Rng;
    let exp_delay = base_ms.saturating_mul(2u64.saturating_pow(attempt - 1));
    let capped = exp_delay.min(max_ms);
    // 添加 0-25% 的随机抖动，避免惊群效应
    let jitter = rand::rng().random_range(0..=capped / 4);
    capped.saturating_add(jitter)
}
