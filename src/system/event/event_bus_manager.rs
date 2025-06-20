use crate::system::event::{EventBus, EventHandler};
use once_cell::sync::Lazy;
use std::sync::Arc;

/// 全局事件总线实例
/// 使用 Lazy 确保线程安全的单例模式
pub static GLOBAL_EVENT_BUS: Lazy<Arc<EventBus>> = Lazy::new(|| {
    Arc::new(EventBus::new(1000)) // 保留最近 1000 个事件的历史记录
});

/// 事件总线管理器
/// 提供便捷的方法来访问和管理全局事件总线
pub struct EventBusManager;

impl EventBusManager {
    /// 获取全局事件总线的引用
    pub fn instance() -> Arc<EventBus> {
        GLOBAL_EVENT_BUS.clone()
    }

    /// 注册事件处理器到全局事件总线
    pub fn register_handler(handler: Arc<dyn EventHandler>) {
        GLOBAL_EVENT_BUS.register_handler(handler);
    }

    /// 清理事件历史（可用于内存管理）
    pub fn clear_history() {
        GLOBAL_EVENT_BUS.clear_history();
    }

    /// 获取事件历史的统计信息
    pub fn get_history_stats() -> (usize, Vec<String>) {
        let history = GLOBAL_EVENT_BUS.get_history();
        let count = history.len();
        let recent_events: Vec<String> = history
            .iter()
            .rev()
            .take(5)
            .map(|e| format!("{:?} from {}", e.event_type, e.source))
            .collect();

        (count, recent_events)
    }
}

/// 便捷宏：快速发布事件
#[macro_export]
macro_rules! publish_event {
    ($event:expr) => {
        if let Err(e) = $crate::system::event::event_bus_manager::GLOBAL_EVENT_BUS
            .publish($event)
            .await
        {
            tracing::error!("Failed to publish event: {}", e);
        }
    };
}

/// 便捷宏：创建并发布短链接创建事件
#[macro_export]
macro_rules! publish_shortlink_created {
    ($code:expr, $target:expr, $source:expr) => {
        $crate::publish_event!($crate::system::event::Event::shortlink_created(
            $code, $target, $source
        ));
    };
}

/// 便捷宏：创建并发布短链接删除事件
#[macro_export]
macro_rules! publish_shortlink_deleted {
    ($code:expr, $source:expr) => {
        $crate::publish_event!($crate::system::event::Event::shortlink_deleted(
            $code, $source
        ));
    };
}

/// 便捷宏：创建并发布短链接更新事件
#[macro_export]
macro_rules! publish_shortlink_updated {
    ($code:expr, $target:expr, $source:expr) => {
        $crate::publish_event!($crate::system::event::Event::shortlink_updated(
            $code, $target, $source
        ));
    };
}

/// 便捷宏：创建并发布短链接访问事件
#[macro_export]
macro_rules! publish_shortlink_accessed {
    ($code:expr, $client_ip:expr, $user_agent:expr, $source:expr) => {
        $crate::publish_event!($crate::system::event::Event::shortlink_accessed(
            $code,
            $client_ip,
            $user_agent,
            $source
        ));
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::event::{Event, EventType};

    #[tokio::test]
    async fn test_global_event_bus() {
        let event_bus = EventBusManager::instance();

        let event = Event::shortlink_created("test", "https://test.com", "test");
        event_bus.publish(event).await.unwrap();

        let (count, _) = EventBusManager::get_history_stats();
        assert!(count > 0);
    }

    #[tokio::test]
    async fn test_convenience_macros() {
        // 测试宏的编译
        let code = "test123";
        let target = "https://example.com";
        let source = "test";

        publish_shortlink_created!(code, target, source);
        publish_shortlink_updated!(code, "https://new-example.com", source);
        publish_shortlink_accessed!(code, Some("127.0.0.1"), Some("test-agent"), source);
        publish_shortlink_deleted!(code, source);

        // 等待事件处理
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let (count, _) = EventBusManager::get_history_stats();
        assert!(count >= 4);
    }
}
