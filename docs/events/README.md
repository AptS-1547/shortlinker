# Shortlinker 事件系统文档

## 概述

Shortlinker 的事件系统是一个强大而灵活的组件，用于处理应用程序中发生的各种事件。该系统基于发布-订阅模式，允许不同的组件响应系统中的状态变化。

## 核心组件

### 1. 事件类型 (EventType)

系统支持以下预定义的事件类型：

- `ShortLinkCreated` - 短链接创建事件
- `ShortLinkDeleted` - 短链接删除事件  
- `ShortLinkUpdated` - 短链接更新事件
- `ShortLinkAccessed` - 短链接访问事件
- `SystemStartup` - 系统启动事件
- `SystemShutdown` - 系统关闭事件
- `ConfigReloaded` - 配置重载事件
- `Custom(String)` - 自定义事件类型

### 2. 事件结构 (Event)

每个事件包含以下信息：

```rust
pub struct Event {
    pub id: String,                    // 唯一标识符
    pub event_type: EventType,         // 事件类型
    pub timestamp: SystemTime,         // 时间戳
    pub payload: EventPayload,         // 事件数据
    pub source: String,                // 事件来源
}
```

### 3. 事件负载 (EventPayload)

事件负载包含与特定事件相关的数据：

- `ShortLink` - 短链接相关数据（代码、目标URL等）
- `System` - 系统相关数据（消息、详细信息等）
- `Access` - 访问相关数据（客户端IP、用户代理等）
- `Custom` - 自定义数据

### 4. 事件处理器 (EventHandler)

事件处理器是实现 `EventHandler` trait 的组件，用于响应特定类型的事件：

```rust
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn name(&self) -> &str;
    fn interested_events(&self) -> Vec<EventType>;
}
```

### 5. 事件总线 (EventBus)

事件总线是系统的核心，负责：

- 管理事件处理器的注册
- 发布事件到所有相关处理器
- 维护事件历史记录
- 提供事件订阅功能

## 使用方法

### 基本使用

#### 1. 初始化事件系统

```rust
use shortlinker::system::{EventBusManager, Event, EventType};

// 初始化默认处理器
EventBusManager::initialize_default_handlers();
```

#### 2. 发布事件

```rust
// 使用便捷方法创建事件
let event = Event::shortlink_created("abc123", "https://example.com", "api_service");

// 发布事件
let event_bus = EventBusManager::instance();
event_bus.publish(event).await?;
```

#### 3. 使用便捷宏

```rust
// 使用宏快速发布事件
publish_shortlink_created!("abc123", "https://example.com", "api_service");
publish_shortlink_accessed!("abc123", Some("192.168.1.1"), Some("Mozilla/5.0..."), "redirect");
publish_shortlink_updated!("abc123", "https://new-url.com", "admin");
publish_shortlink_deleted!("abc123", "admin");
```

### 创建自定义事件处理器

```rust
use async_trait::async_trait;
use shortlinker::system::{EventHandler, Event, EventType};

#[derive(Debug)]
pub struct MyCustomHandler {
    name: String,
}

impl MyCustomHandler {
    pub fn new() -> Self {
        Self {
            name: "my_custom_handler".to_string(),
        }
    }
}

#[async_trait]
impl EventHandler for MyCustomHandler {
    async fn handle(&self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match &event.event_type {
            EventType::ShortLinkCreated => {
                // 处理短链接创建逻辑
                println!("处理短链接创建: {:?}", event.payload);
            },
            EventType::ShortLinkAccessed => {
                // 处理访问统计逻辑
                println!("记录访问统计: {:?}", event.payload);
            },
            _ => {}
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn interested_events(&self) -> Vec<EventType> {
        vec![
            EventType::ShortLinkCreated,
            EventType::ShortLinkAccessed,
        ]
    }
}

// 注册自定义处理器
EventBusManager::register_handler(Arc::new(MyCustomHandler::new()));
```

### 订阅事件流

```rust
let event_bus = EventBusManager::instance();
let mut receiver = event_bus.subscribe();

// 在后台任务中监听事件
tokio::spawn(async move {
    while let Ok(event) = receiver.recv().await {
        println!("收到实时事件: {:?}", event.event_type);
    }
});
```

### 查询事件历史

```rust
let event_bus = EventBusManager::instance();

// 获取所有历史事件
let all_history = event_bus.get_history();

// 获取特定类型的历史事件
let access_events = event_bus.get_history_by_type(&EventType::ShortLinkAccessed);

// 获取历史统计信息
let (count, recent_events) = EventBusManager::get_history_stats();
```

## 内置事件处理器

### 1. LoggingHandler

将所有事件记录到应用程序日志中，便于调试和监控。

### 2. StatisticsHandler

统计各类事件的数量，提供系统使用情况的洞察。

### 3. CacheInvalidationHandler

当短链接发生变化时自动清理相关缓存，确保数据一致性。

## 最佳实践

### 1. 事件命名

- 使用清晰、描述性的事件名称
- 遵循一致的命名约定
- 为自定义事件使用有意义的标识符

### 2. 错误处理

- 事件处理器应该处理错误但不应该抛出异常
- 使用日志记录处理失败的情况
- 考虑实现重试机制

### 3. 性能考虑

- 避免在事件处理器中执行耗时操作
- 考虑使用异步处理来避免阻塞
- 定期清理事件历史以控制内存使用

### 4. 测试

- 为自定义事件处理器编写单元测试
- 测试事件的发布和处理流程
- 验证事件负载的正确性

## 配置选项

### 事件历史大小

```rust
// 创建事件总线时指定历史记录大小
let event_bus = EventBus::new(5000); // 保留最近 5000 个事件
```

### 自定义事件总线

```rust
// 如果需要独立的事件总线实例
let custom_event_bus = Arc::new(EventBus::new(1000));
custom_event_bus.register_handler(Arc::new(MyHandler::new()));
```

## 扩展示例

### 1. 集成外部服务

```rust
#[derive(Debug)]
pub struct WebhookHandler {
    webhook_url: String,
    client: reqwest::Client,
}

#[async_trait]
impl EventHandler for WebhookHandler {
    async fn handle(&self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 将事件发送到外部 webhook
        let payload = serde_json::to_string(event)?;
        self.client.post(&self.webhook_url)
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .await?;
        Ok(())
    }

    fn name(&self) -> &str {
        "webhook_handler"
    }

    fn interested_events(&self) -> Vec<EventType> {
        vec![EventType::ShortLinkCreated, EventType::ShortLinkDeleted]
    }
}
```

### 2. 数据库审计

```rust
#[derive(Debug)]
pub struct AuditHandler {
    db_pool: sqlx::Pool<sqlx::Sqlite>,
}

#[async_trait]
impl EventHandler for AuditHandler {
    async fn handle(&self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 将事件保存到审计表
        sqlx::query!(
            "INSERT INTO audit_log (event_id, event_type, timestamp, payload, source) VALUES (?, ?, ?, ?, ?)",
            event.id,
            format!("{:?}", event.event_type),
            event.timestamp,
            serde_json::to_string(&event.payload)?,
            event.source
        )
        .execute(&self.db_pool)
        .await?;
        Ok(())
    }

    fn name(&self) -> &str {
        "audit_handler"
    }

    fn interested_events(&self) -> Vec<EventType> {
        // 审计所有事件
        vec![
            EventType::ShortLinkCreated,
            EventType::ShortLinkDeleted,
            EventType::ShortLinkUpdated,
            EventType::ShortLinkAccessed,
        ]
    }
}
```

## 故障排查

### 常见问题

1. **事件处理器未被调用**
   - 检查处理器是否正确注册
   - 验证 `interested_events()` 方法返回正确的事件类型

2. **事件丢失**
   - 检查事件总线的容量设置
   - 确保在发布事件前已注册处理器

3. **性能问题**
   - 检查事件处理器是否执行耗时操作
   - 考虑使用异步处理或后台任务

### 调试建议

- 启用详细日志记录
- 使用 `LoggingHandler` 查看所有事件流
- 监控事件历史大小和处理延迟

## 结论

Shortlinker 的事件系统提供了一个强大且灵活的机制来处理应用程序中的各种事件。通过合理使用事件处理器和遵循最佳实践，可以构建出高度可扩展和可维护的应用程序架构。
