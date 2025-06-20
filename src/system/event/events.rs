use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{warn, error};

/// 事件类型枚举，定义系统中可能发生的各种事件
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// 短链接创建事件
    ShortLinkCreated,
    /// 短链接删除事件
    ShortLinkDeleted,
    /// 短链接更新事件
    ShortLinkUpdated,
    /// 短链接访问事件
    ShortLinkAccessed,
    /// 系统启动事件
    SystemStartup,
    /// 系统关闭事件
    SystemShutdown,
    /// 配置重载事件
    ConfigReloaded,
    /// 自定义事件
    Custom(String),
}

/// 事件数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// 事件唯一标识符
    pub id: String,
    /// 事件类型
    pub event_type: EventType,
    /// 事件时间戳
    pub timestamp: SystemTime,
    /// 事件数据负载
    pub payload: EventPayload,
    /// 事件来源
    pub source: String,
}

/// 事件负载数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    /// 短链接相关数据
    ShortLink {
        code: String,
        target: Option<String>,
        created_at: Option<String>,
        expires_at: Option<String>,
    },
    /// 系统相关数据
    System {
        message: String,
        details: Option<HashMap<String, String>>,
    },
    /// 访问相关数据
    Access {
        code: String,
        client_ip: Option<String>,
        user_agent: Option<String>,
    },
    /// 自定义数据
    Custom(HashMap<String, String>),
}

/// 事件处理器特征
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// 处理事件的异步方法
    async fn handle(&self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取处理器名称
    fn name(&self) -> &str;
    
    /// 获取感兴趣的事件类型
    fn interested_events(&self) -> Vec<EventType>;
}

/// 事件总线，负责管理事件的发布和订阅
pub struct EventBus {
    /// 事件处理器存储
    handlers: Arc<Mutex<HashMap<EventType, Vec<Arc<dyn EventHandler>>>>>,
    /// 广播发送器
    sender: broadcast::Sender<Event>,
    /// 事件历史记录（可选）
    history: Arc<Mutex<Vec<Event>>>,
    /// 最大历史记录数量
    max_history: usize,
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new(max_history: usize) -> Self {
        let (sender, _) = broadcast::channel(1000);
        
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            sender,
            history: Arc::new(Mutex::new(Vec::new())),
            max_history,
        }
    }

    /// 注册事件处理器
    pub fn register_handler(&self, handler: Arc<dyn EventHandler>) {
        let mut handlers = self.handlers.lock().unwrap();
        
        for event_type in handler.interested_events() {
            handlers
                .entry(event_type)
                .or_insert_with(Vec::new)
                .push(handler.clone());
        }
    }

    /// 发布事件
    pub async fn publish(&self, event: Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 添加到历史记录
        {
            let mut history = self.history.lock().unwrap();
            history.push(event.clone());
            
            // 保持历史记录在限制范围内
            if history.len() > self.max_history {
                history.remove(0);
            }
        }

        // 通过广播通道发送事件
        if let Err(e) = self.sender.send(event.clone()) {
            warn!("Failed to broadcast event: {}", e);
        }

        // 调用相关的事件处理器
        let handlers = self.handlers.lock().unwrap();
        if let Some(event_handlers) = handlers.get(&event.event_type) {
            for handler in event_handlers {
                if let Err(e) = handler.handle(&event).await {
                    error!("Event handler '{}' failed: {}", handler.name(), e);
                }
            }
        }

        Ok(())
    }

    /// 订阅事件流
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    /// 获取事件历史
    pub fn get_history(&self) -> Vec<Event> {
        self.history.lock().unwrap().clone()
    }

    /// 获取指定类型的事件历史
    pub fn get_history_by_type(&self, event_type: &EventType) -> Vec<Event> {
        self.history
            .lock()
            .unwrap()
            .iter()
            .filter(|event| &event.event_type == event_type)
            .cloned()
            .collect()
    }

    /// 清空事件历史
    pub fn clear_history(&self) {
        self.history.lock().unwrap().clear();
    }
}

/// 事件构建器，方便创建事件
pub struct EventBuilder {
    event_type: EventType,
    source: String,
    payload: Option<EventPayload>,
}

impl EventBuilder {
    /// 创建新的事件构建器
    pub fn new(event_type: EventType, source: &str) -> Self {
        Self {
            event_type,
            source: source.to_string(),
            payload: None,
        }
    }

    /// 设置事件负载
    pub fn with_payload(mut self, payload: EventPayload) -> Self {
        self.payload = Some(payload);
        self
    }

    /// 构建事件
    pub fn build(self) -> Event {
        Event {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: self.event_type,
            timestamp: SystemTime::now(),
            payload: self.payload.unwrap_or(EventPayload::Custom(HashMap::new())),
            source: self.source,
        }
    }
}

/// 便捷的事件创建函数
impl Event {
    /// 创建短链接创建事件
    pub fn shortlink_created(code: &str, target: &str, source: &str) -> Self {
        EventBuilder::new(EventType::ShortLinkCreated, source)
            .with_payload(EventPayload::ShortLink {
                code: code.to_string(),
                target: Some(target.to_string()),
                created_at: Some(chrono::Utc::now().to_rfc3339()),
                expires_at: None,
            })
            .build()
    }

    /// 创建短链接删除事件
    pub fn shortlink_deleted(code: &str, source: &str) -> Self {
        EventBuilder::new(EventType::ShortLinkDeleted, source)
            .with_payload(EventPayload::ShortLink {
                code: code.to_string(),
                target: None,
                created_at: None,
                expires_at: None,
            })
            .build()
    }

    /// 创建短链接更新事件
    pub fn shortlink_updated(code: &str, new_target: &str, source: &str) -> Self {
        EventBuilder::new(EventType::ShortLinkUpdated, source)
            .with_payload(EventPayload::ShortLink {
                code: code.to_string(),
                target: Some(new_target.to_string()),
                created_at: None,
                expires_at: None,
            })
            .build()
    }

    /// 创建短链接访问事件
    pub fn shortlink_accessed(code: &str, client_ip: Option<&str>, user_agent: Option<&str>, source: &str) -> Self {
        EventBuilder::new(EventType::ShortLinkAccessed, source)
            .with_payload(EventPayload::Access {
                code: code.to_string(),
                client_ip: client_ip.map(|s| s.to_string()),
                user_agent: user_agent.map(|s| s.to_string()),
            })
            .build()
    }

    /// 创建系统事件
    pub fn system_event(event_type: EventType, message: &str, source: &str) -> Self {
        EventBuilder::new(event_type, source)
            .with_payload(EventPayload::System {
                message: message.to_string(),
                details: None,
            })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct TestHandler {
        name: String,
        counter: Arc<AtomicUsize>,
        interested_events: Vec<EventType>,
    }

    impl TestHandler {
        fn new(name: &str, interested_events: Vec<EventType>) -> Self {
            Self {
                name: name.to_string(),
                counter: Arc::new(AtomicUsize::new(0)),
                interested_events,
            }
        }

        fn get_count(&self) -> usize {
            self.counter.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl EventHandler for TestHandler {
        async fn handle(&self, _event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn interested_events(&self) -> Vec<EventType> {
            self.interested_events.clone()
        }
    }

    #[tokio::test]
    async fn test_event_bus() {
        let event_bus = EventBus::new(100);
        
        let handler = Arc::new(TestHandler::new(
            "test_handler",
            vec![EventType::ShortLinkCreated, EventType::ShortLinkDeleted],
        ));
        
        event_bus.register_handler(handler.clone());

        // 测试发布事件
        let event = Event::shortlink_created("test123", "https://example.com", "test");
        event_bus.publish(event).await.unwrap();

        // 验证处理器被调用
        assert_eq!(handler.get_count(), 1);

        // 测试历史记录
        let history = event_bus.get_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].event_type, EventType::ShortLinkCreated);
    }
}
