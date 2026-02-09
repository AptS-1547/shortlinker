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
    match err {
        DbErr::ConnectionAcquire(_) | // 连接池获取失败
        DbErr::Conn(_) => true, // 连接问题
        DbErr::Exec(runtime_err) | DbErr::Query(runtime_err) => {
            is_retryable_runtime_error(runtime_err)
        }
        _ => false,
    }
}

/// 判断运行时错误是否可重试（死锁、锁超时等）
fn is_retryable_runtime_error(err: &sea_orm::error::RuntimeErr) -> bool {
    use sea_orm::error::RuntimeErr;

    match err {
        RuntimeErr::SqlxError(sqlx_err) => {
            use std::ops::Deref;
            if let Some(db_err) = sqlx_err.deref().as_database_error() {
                // 通过错误码识别可重试错误
                if let Some(code) = db_err.code() {
                    let code_str = code.as_ref();
                    return matches!(
                        code_str,
                        // MySQL 死锁和锁超时
                        "1213" | "1205" |
                        // PostgreSQL 序列化失败和死锁
                        "40001" | "40P01" |
                        // SQLite BUSY 和 LOCKED
                        "5" | "6"
                    );
                }
            }
            // 回退到字符串匹配（用于非 Database 错误）
            let err_str = sqlx_err.to_string().to_lowercase();
            is_retryable_error_message(&err_str)
        }
        RuntimeErr::Internal(msg) => {
            let err_str = msg.to_lowercase();
            is_retryable_error_message(&err_str)
        }
        #[allow(unreachable_patterns)]
        _ => false,
    }
}

/// 通过错误消息判断是否可重试（回退方案）
fn is_retryable_error_message(err_str: &str) -> bool {
    err_str.contains("deadlock")
        || err_str.contains("lock wait timeout")
        || err_str.contains("database is locked")
        || err_str.contains("serialization failure")
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
                    debug!(
                        "Operation '{}' succeeded after {} retries",
                        operation_name, attempt
                    );
                }
                return Ok(result);
            }
            Err(e) if is_retryable_error(&e) && attempt < config.max_retries => {
                attempt += 1;
                let delay = calculate_backoff(attempt, config.base_delay_ms, config.max_delay_ms);
                warn!(
                    "Operation '{}' failed (attempt {}/{}): {}; retrying in {} ms",
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
                    debug!(
                        "Operation '{}' failed with non-retryable error: {}",
                        operation_name, e
                    );
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

/// 带超时的重试执行器
///
/// 与 `with_retry` 相同，但每次操作有独立的超时控制
pub async fn with_retry_timeout<T, F, Fut>(
    operation_name: &str,
    config: RetryConfig,
    timeout_ms: u64,
    mut operation: F,
) -> Result<T, DbErr>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, DbErr>>,
{
    let mut attempt = 0;
    loop {
        let timeout_duration = Duration::from_millis(timeout_ms);
        let result = tokio::time::timeout(timeout_duration, operation()).await;

        match result {
            Ok(Ok(value)) => {
                if attempt > 0 {
                    debug!(
                        "Operation '{}' succeeded after {} retries",
                        operation_name, attempt
                    );
                }
                return Ok(value);
            }
            Ok(Err(e)) if is_retryable_error(&e) && attempt < config.max_retries => {
                attempt += 1;
                let delay = calculate_backoff(attempt, config.base_delay_ms, config.max_delay_ms);
                warn!(
                    "Operation '{}' failed (attempt {}/{}): {}; retrying in {} ms",
                    operation_name,
                    attempt,
                    config.max_retries + 1,
                    e,
                    delay
                );
                sleep(Duration::from_millis(delay)).await;
            }
            Ok(Err(e)) => {
                if !is_retryable_error(&e) {
                    debug!(
                        "Operation '{}' failed with non-retryable error: {}",
                        operation_name, e
                    );
                }
                return Err(e);
            }
            Err(_elapsed) => {
                // 超时
                if attempt < config.max_retries {
                    attempt += 1;
                    let delay =
                        calculate_backoff(attempt, config.base_delay_ms, config.max_delay_ms);
                    warn!(
                        "Operation '{}' timed out after {}ms (attempt {}/{}); retrying in {} ms",
                        operation_name,
                        timeout_ms,
                        attempt,
                        config.max_retries + 1,
                        delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                } else {
                    warn!(
                        "Operation '{}' timed out after {}ms, retries exhausted",
                        operation_name, timeout_ms
                    );
                    return Err(DbErr::Custom(format!(
                        "Operation '{}' timed out after {}ms",
                        operation_name, timeout_ms
                    )));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_is_retryable_error_connection_acquire() {
        let err = DbErr::ConnectionAcquire(sea_orm::error::ConnAcquireErr::Timeout);
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn test_is_retryable_error_conn() {
        let err = DbErr::Conn(sea_orm::error::RuntimeErr::Internal(
            "connection lost".to_string(),
        ));
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn test_is_retryable_error_record_not_found() {
        let err = DbErr::RecordNotFound("not found".to_string());
        assert!(!is_retryable_error(&err));
    }

    #[test]
    fn test_is_retryable_error_deadlock() {
        // 测试死锁错误应该被重试
        let err = DbErr::Exec(sea_orm::error::RuntimeErr::Internal(
            "Deadlock found when trying to get lock".to_string(),
        ));
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn test_is_retryable_error_database_locked() {
        // 测试 SQLite 数据库锁定错误应该被重试
        let err = DbErr::Query(sea_orm::error::RuntimeErr::Internal(
            "database is locked".to_string(),
        ));
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn test_is_retryable_error_lock_wait_timeout() {
        // 测试锁等待超时应该被重试
        let err = DbErr::Exec(sea_orm::error::RuntimeErr::Internal(
            "Lock wait timeout exceeded".to_string(),
        ));
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn test_calculate_backoff_exponential() {
        // 第一次重试：base_ms * 2^0 = 100
        let delay1 = calculate_backoff(1, 100, 2000);
        assert!((100..=125).contains(&delay1)); // 100 + 0-25% jitter

        // 第二次重试：base_ms * 2^1 = 200
        let delay2 = calculate_backoff(2, 100, 2000);
        assert!((200..=250).contains(&delay2));

        // 第三次重试：base_ms * 2^2 = 400
        let delay3 = calculate_backoff(3, 100, 2000);
        assert!((400..=500).contains(&delay3));
    }

    #[test]
    fn test_calculate_backoff_capped_at_max() {
        // 第 10 次重试会超过 max，应该被限制
        let delay = calculate_backoff(10, 100, 2000);
        assert!((2000..=2500).contains(&delay)); // 2000 + 0-25% jitter
    }

    #[tokio::test]
    async fn test_with_retry_success_first_try() {
        let config = RetryConfig::default();
        let call_count = AtomicU32::new(0);

        let result = with_retry("test_op", config, || {
            call_count.fetch_add(1, Ordering::SeqCst);
            async { Ok::<_, DbErr>(42) }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_with_retry_success_after_retries() {
        let config = RetryConfig {
            max_retries: 3,
            base_delay_ms: 10, // 短延迟加速测试
            max_delay_ms: 50,
        };
        let call_count = AtomicU32::new(0);

        let result = with_retry("test_op", config, || {
            let count = call_count.fetch_add(1, Ordering::SeqCst);
            async move {
                if count < 2 {
                    Err(DbErr::ConnectionAcquire(
                        sea_orm::error::ConnAcquireErr::Timeout,
                    ))
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 3); // 初始 + 2 次重试
    }

    #[tokio::test]
    async fn test_with_retry_exhausted() {
        let config = RetryConfig {
            max_retries: 2,
            base_delay_ms: 10,
            max_delay_ms: 50,
        };
        let call_count = AtomicU32::new(0);

        let result = with_retry("test_op", config, || {
            call_count.fetch_add(1, Ordering::SeqCst);
            async {
                Err::<i32, _>(DbErr::ConnectionAcquire(
                    sea_orm::error::ConnAcquireErr::Timeout,
                ))
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 3); // 初始 + 2 次重试
    }

    #[tokio::test]
    async fn test_with_retry_non_retryable_error_no_retry() {
        let config = RetryConfig::default();
        let call_count = AtomicU32::new(0);

        let result = with_retry("test_op", config, || {
            call_count.fetch_add(1, Ordering::SeqCst);
            async { Err::<i32, _>(DbErr::RecordNotFound("not found".to_string())) }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // 不可重试，只调用一次
    }
}
