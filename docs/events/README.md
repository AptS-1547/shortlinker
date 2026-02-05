# Shortlinker 事件系统

## 概述

事件系统是 Shortlinker 的核心组件，用于解耦系统各部分之间的通信。基于 `tokio::sync::broadcast` 实现发布-订阅模式。

### 设计原则

1. **扩展而非重写**：基于现有 `ReloadCoordinator` 扩展
2. **简单优先**：不做过度抽象
3. **按需实现**：处理器按实际需求添加

## 事件类型

所有事件统一定义在 `SystemEvent` 枚举中：

```rust
#[derive(Debug, Clone)]
pub enum SystemEvent {
    // === 生命周期事件 ===
    /// 系统启动完成
    Startup,
    /// 系统关闭
    Shutdown { graceful: bool },

    // === 配置/数据重载 ===
    /// 重载开始
    ReloadStarted { target: ReloadTarget },
    /// 重载完成
    ReloadCompleted { target: ReloadTarget, duration_ms: u64 },
    /// 重载失败
    ReloadFailed { target: ReloadTarget, error: String },

    // === 链接事件 ===
    /// 链接创建
    LinkCreated { code: String },
    /// 链接更新
    LinkUpdated { code: String },
    /// 链接删除
    LinkDeleted { code: String },
    /// 批量导入完成
    BulkImport { count: usize, success: usize },

    // === 配置变更 ===
    /// 运行时配置变更
    ConfigChanged { key: String },
}
```

### ReloadTarget

```rust
#[derive(Debug, Clone, Copy)]
pub enum ReloadTarget {
    /// 重载所有（配置 + 数据）
    All,
    /// 仅重载配置
    Config,
    /// 仅重载数据（存储 + 缓存）
    Data,
}
```

## 核心组件

### SystemEventBus

事件总线，负责事件的发布和订阅。

```rust
pub struct SystemEventBus {
    sender: broadcast::Sender<SystemEvent>,
}

impl SystemEventBus {
    /// 创建新的事件总线
    pub fn new(capacity: usize) -> Self;

    /// 发布事件
    pub fn publish(&self, event: SystemEvent);

    /// 订阅事件流
    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent>;
}
```

### 全局实例

```rust
/// 获取全局事件总线实例
pub fn event_bus() -> &'static SystemEventBus;

/// 初始化事件总线（在 startup 时调用）
pub fn init_event_bus(capacity: usize);
```

## 使用方法

### 发布事件

```rust
use crate::system::events::{event_bus, SystemEvent};

// 在 LinkService 中发布链接创建事件
event_bus().publish(SystemEvent::LinkCreated {
    code: "abc123".to_string(),
});

// 批量导入完成
event_bus().publish(SystemEvent::BulkImport {
    count: 100,
    success: 98,
});
```

### 订阅事件

```rust
use crate::system::events::{event_bus, SystemEvent};

// 在后台任务中订阅事件
let mut receiver = event_bus().subscribe();

tokio::spawn(async move {
    while let Ok(event) = receiver.recv().await {
        match event {
            SystemEvent::LinkCreated { code } => {
                println!("链接创建: {}", code);
            }
            SystemEvent::Shutdown { graceful } => {
                println!("系统关闭 (graceful: {})", graceful);
                break;
            }
            _ => {}
        }
    }
});
```

## 事件处理器

### EventHandler Trait

```rust
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// 处理事件
    async fn handle(&self, event: &SystemEvent);

    /// 处理器名称（用于日志）
    fn name(&self) -> &str;
}
```

### 创建自定义处理器

```rust
use async_trait::async_trait;
use crate::system::events::{EventHandler, SystemEvent};

pub struct WebhookHandler {
    client: reqwest::Client,
    webhook_url: String,
}

#[async_trait]
impl EventHandler for WebhookHandler {
    async fn handle(&self, event: &SystemEvent) {
        // 只处理链接事件
        match event {
            SystemEvent::LinkCreated { code } => {
                self.send_webhook("link_created", code).await;
            }
            SystemEvent::LinkDeleted { code } => {
                self.send_webhook("link_deleted", code).await;
            }
            _ => {} // 忽略其他事件
        }
    }

    fn name(&self) -> &str {
        "webhook_handler"
    }
}
```

### 注册处理器

```rust
use crate::system::events::event_bus;

// 在启动时注册处理器
let handler = Arc::new(WebhookHandler::new(webhook_url));
let mut receiver = event_bus().subscribe();

tokio::spawn(async move {
    while let Ok(event) = receiver.recv().await {
        handler.handle(&event).await;
    }
});
```

## 内置处理器

### LoggingHandler

将事件记录到日志系统。

```rust
pub struct LoggingHandler;

#[async_trait]
impl EventHandler for LoggingHandler {
    async fn handle(&self, event: &SystemEvent) {
        match event {
            SystemEvent::LinkCreated { code } => {
                info!(code = %code, "Link created");
            }
            SystemEvent::LinkDeleted { code } => {
                info!(code = %code, "Link deleted");
            }
            SystemEvent::ReloadCompleted { target, duration_ms } => {
                info!(?target, duration_ms, "Reload completed");
            }
            _ => {
                debug!(?event, "Event received");
            }
        }
    }

    fn name(&self) -> &str {
        "logging_handler"
    }
}
```

## 注意事项

### Broadcast Channel 特性

1. **容量限制**：创建时指定容量（默认 256），超出会丢弃旧消息
2. **Lagged 错误**：消费者处理太慢会收到 `RecvError::Lagged`
3. **Clone 要求**：事件必须实现 `Clone`

### 性能考虑

1. **避免在热路径发布事件**：重定向处理（`redirect.rs`）不应发布事件
2. **事件数据要小**：只包含必要信息（如 code），不要包含整个对象
3. **处理器要快**：耗时操作应该 spawn 新任务

### 错误处理

```rust
// 发布事件不会失败（无接收者时静默丢弃）
event_bus().publish(event);

// 接收可能返回 Lagged 错误
match receiver.recv().await {
    Ok(event) => { /* 处理事件 */ }
    Err(broadcast::error::RecvError::Lagged(n)) => {
        warn!("Missed {} events", n);
    }
    Err(broadcast::error::RecvError::Closed) => {
        break; // 发送端关闭
    }
}
```

## 模块结构

```
src/system/events/
├── mod.rs              # 模块导出
├── types.rs            # SystemEvent, ReloadTarget 定义
├── bus.rs              # SystemEventBus 实现
├── coordinator.rs      # ReloadCoordinator（重载功能）
├── global.rs           # 全局实例管理
└── handlers/
    ├── mod.rs
    └── logging.rs      # LoggingHandler
```

## 与 ReloadCoordinator 的关系

`ReloadCoordinator` 是事件系统的特化应用，专门处理配置和数据重载：

```rust
pub trait ReloadCoordinator: Send + Sync {
    /// 执行重载
    async fn reload(&self, target: ReloadTarget) -> Result<ReloadResult>;

    /// 获取重载状态
    fn status(&self) -> ReloadStatus;

    /// 订阅重载事件（返回 SystemEvent 的子集）
    fn subscribe(&self) -> broadcast::Receiver<SystemEvent>;
}
```

`ReloadCoordinator` 在执行重载时会发布 `ReloadStarted`、`ReloadCompleted` 或 `ReloadFailed` 事件到 `SystemEventBus`。

## FAQ

### Q: 为什么不用 `mpsc` channel？

A: `mpsc` 是多生产者单消费者，而我们需要多个处理器同时收到事件。`broadcast` 支持多消费者。

### Q: 为什么没有事件历史？

A: 事件历史增加内存开销且实际需求不明确。日志系统已经记录了所有重要事件。

### Q: 为什么处理器没有 `interested_events()` 方法？

A: 在 `handle()` 中用 `match` 过滤更简单直接，不需要额外的元数据。
