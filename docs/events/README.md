# Shortlinker 事件系统

## 阅读路径建议

- **快速理解（5 分钟）**：先看 [概述](#概述) → [核心概念](#核心概念) → [内置事件](#内置事件)
- **要开始开发处理器**：继续看 [使用示例](#使用示例) → [事件命名约定](#事件命名约定)
- **要做性能与稳定性评估**：看 [性能考虑](#性能考虑) → [FAQ](#faq)

## 概述

事件系统是 Shortlinker 的核心基础设施，为后续插件系统提供支撑。基于 `tokio::sync::broadcast` 实现发布-订阅模式，支持：

- **自定义事件**：插件可定义和发布自己的事件类型
- **事件拦截**：处理器可取消可取消事件的传播
- **优先级控制**：多个处理器按优先级顺序执行

### 设计原则

1. **类型安全**：使用 trait 而非 enum，支持强类型自定义事件
2. **插件友好**：插件可以定义、发布、订阅事件
3. **可控传播**：支持事件取消和优先级

## 核心概念

### Event Trait

所有事件必须实现 `Event` trait：

```rust
pub trait Event: Send + Sync + Clone + 'static {
    /// 事件名称（用于调试/日志/订阅过滤）
    fn name(&self) -> &'static str;

    /// 是否可取消（默认不可取消）
    fn cancellable(&self) -> bool {
        false
    }
}
```

### EventContext

事件上下文包装事件实例，提供取消机制：

```rust
pub struct EventContext<E: Event> {
    event: E,
    cancelled: bool,
    cancel_reason: Option<String>,
}

impl<E: Event> EventContext<E> {
    /// 获取事件引用
    pub fn event(&self) -> &E;

    /// 获取事件可变引用（允许修改事件数据）
    pub fn event_mut(&mut self) -> &mut E;

    /// 取消事件（仅对 cancellable 事件有效）
    pub fn cancel(&mut self, reason: impl Into<String>) -> bool;

    /// 检查是否已取消
    pub fn is_cancelled(&self) -> bool;

    /// 获取取消原因
    pub fn cancel_reason(&self) -> Option<&str>;
}
```

### Priority

处理器优先级，决定执行顺序：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// 最先执行（用于前置校验）
    Highest = 0,
    /// 高优先级
    High = 1,
    /// 默认优先级
    Normal = 2,
    /// 低优先级
    Low = 3,
    /// 最后执行
    Lowest = 4,
    /// 监控模式：最后执行，不能取消事件，用于日志/统计
    Monitor = 5,
}
```

**执行顺序**：Highest → High → Normal → Low → Lowest → Monitor

**Monitor 特殊规则**：
- 总是最后执行
- 不能调用 `cancel()`
- 即使事件被取消也会收到通知（可用于审计日志）

## 内置事件

### 生命周期事件

```rust
/// 系统启动完成
#[derive(Debug, Clone)]
pub struct StartupEvent {
    pub timestamp: DateTime<Utc>,
}

impl Event for StartupEvent {
    fn name(&self) -> &'static str { "system.startup" }
}

/// 系统关闭
#[derive(Debug, Clone)]
pub struct ShutdownEvent {
    pub graceful: bool,
}

impl Event for ShutdownEvent {
    fn name(&self) -> &'static str { "system.shutdown" }
}
```

### 链接事件

```rust
/// 链接创建前（可取消）
#[derive(Debug, Clone)]
pub struct LinkCreatingEvent {
    pub code: String,
    pub target_url: String,
    pub created_by: Option<String>,
}

impl Event for LinkCreatingEvent {
    fn name(&self) -> &'static str { "link.creating" }
    fn cancellable(&self) -> bool { true }
}

/// 链接创建后（不可取消）
#[derive(Debug, Clone)]
pub struct LinkCreatedEvent {
    pub code: String,
    pub target_url: String,
}

impl Event for LinkCreatedEvent {
    fn name(&self) -> &'static str { "link.created" }
}

/// 链接更新前（可取消）
#[derive(Debug, Clone)]
pub struct LinkUpdatingEvent {
    pub code: String,
    pub old_target_url: String,
    pub new_target_url: String,
}

impl Event for LinkUpdatingEvent {
    fn name(&self) -> &'static str { "link.updating" }
    fn cancellable(&self) -> bool { true }
}

/// 链接更新后
#[derive(Debug, Clone)]
pub struct LinkUpdatedEvent {
    pub code: String,
    pub target_url: String,
}

impl Event for LinkUpdatedEvent {
    fn name(&self) -> &'static str { "link.updated" }
}

/// 链接删除前（可取消）
#[derive(Debug, Clone)]
pub struct LinkDeletingEvent {
    pub code: String,
}

impl Event for LinkDeletingEvent {
    fn name(&self) -> &'static str { "link.deleting" }
    fn cancellable(&self) -> bool { true }
}

/// 链接删除后
#[derive(Debug, Clone)]
pub struct LinkDeletedEvent {
    pub code: String,
}

impl Event for LinkDeletedEvent {
    fn name(&self) -> &'static str { "link.deleted" }
}

/// 批量导入完成
#[derive(Debug, Clone)]
pub struct BulkImportEvent {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
}

impl Event for BulkImportEvent {
    fn name(&self) -> &'static str { "link.bulk_import" }
}
```

### 重载事件

```rust
/// 重载目标
#[derive(Debug, Clone, Copy)]
pub enum ReloadTarget {
    All,
    Config,
    Data,
}

/// 重载开始
#[derive(Debug, Clone)]
pub struct ReloadStartedEvent {
    pub target: ReloadTarget,
}

impl Event for ReloadStartedEvent {
    fn name(&self) -> &'static str { "system.reload.started" }
}

/// 重载完成
#[derive(Debug, Clone)]
pub struct ReloadCompletedEvent {
    pub target: ReloadTarget,
    pub duration_ms: u64,
}

impl Event for ReloadCompletedEvent {
    fn name(&self) -> &'static str { "system.reload.completed" }
}

/// 重载失败
#[derive(Debug, Clone)]
pub struct ReloadFailedEvent {
    pub target: ReloadTarget,
    pub error: String,
}

impl Event for ReloadFailedEvent {
    fn name(&self) -> &'static str { "system.reload.failed" }
}
```

### 配置事件

```rust
/// 配置变更
#[derive(Debug, Clone)]
pub struct ConfigChangedEvent {
    pub key: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

impl Event for ConfigChangedEvent {
    fn name(&self) -> &'static str { "config.changed" }
}
```

## EventBus

### 结构

```rust
pub struct EventBus {
    // 内部实现：类型擦除的处理器注册表
    handlers: RwLock<HashMap<TypeId, Vec<HandlerEntry>>>,
    // broadcast channel 用于异步通知
    sender: broadcast::Sender<Arc<dyn Any + Send + Sync>>,
}

struct HandlerEntry {
    priority: Priority,
    handler: Arc<dyn ErasedHandler>,
}
```

### API

```rust
impl EventBus {
    /// 创建新的事件总线
    pub fn new(capacity: usize) -> Self;

    /// 注册事件处理器
    pub fn register<E, H>(&self, priority: Priority, handler: H)
    where
        E: Event,
        H: EventHandler<E> + 'static;

    /// 发布事件（同步，按优先级执行处理器）
    /// 返回事件是否被取消
    pub async fn publish<E: Event>(&self, event: E) -> EventResult<E>;

    /// 发布事件并忽略结果（fire-and-forget）
    pub fn fire<E: Event>(&self, event: E);

    /// 订阅所有事件的原始流（用于调试/监控）
    pub fn subscribe_all(&self) -> broadcast::Receiver<Arc<dyn Any + Send + Sync>>;
}

/// 事件发布结果
pub struct EventResult<E: Event> {
    pub event: E,
    pub cancelled: bool,
    pub cancel_reason: Option<String>,
}
```

### 全局实例

```rust
/// 获取全局事件总线
pub fn event_bus() -> &'static EventBus;

/// 初始化事件总线（启动时调用一次）
pub fn init_event_bus(capacity: usize);
```

## EventHandler Trait

```rust
#[async_trait]
pub trait EventHandler<E: Event>: Send + Sync {
    /// 处理事件
    async fn handle(&self, ctx: &mut EventContext<E>);

    /// 处理器名称（用于日志）
    fn name(&self) -> &'static str;
}
```

## 使用示例

### 发布事件

```rust
use crate::system::events::{event_bus, LinkCreatingEvent};

// 在 LinkService 中
async fn create_link(&self, code: &str, target_url: &str) -> Result<Link> {
    // 发布创建前事件（可被取消）
    let result = event_bus().publish(LinkCreatingEvent {
        code: code.to_string(),
        target_url: target_url.to_string(),
        created_by: None,
    }).await;

    // 检查是否被取消
    if result.cancelled {
        return Err(Error::Cancelled(result.cancel_reason.unwrap_or_default()));
    }

    // 执行实际创建逻辑
    let link = self.storage.create_link(code, target_url).await?;

    // 发布创建后事件（不可取消，fire-and-forget）
    event_bus().fire(LinkCreatedEvent {
        code: code.to_string(),
        target_url: target_url.to_string(),
    });

    Ok(link)
}
```

### 注册处理器

```rust
use crate::system::events::{event_bus, EventHandler, EventContext, Priority};

// 定义处理器
pub struct UrlValidatorHandler;

#[async_trait]
impl EventHandler<LinkCreatingEvent> for UrlValidatorHandler {
    async fn handle(&self, ctx: &mut EventContext<LinkCreatingEvent>) {
        let url = &ctx.event().target_url;

        // 检查 URL 是否在黑名单
        if is_blacklisted(url) {
            ctx.cancel("URL is blacklisted");
            return;
        }

        // 可以修改事件数据
        if !url.starts_with("https://") {
            ctx.event_mut().target_url = format!("https://{}", url);
        }
    }

    fn name(&self) -> &'static str {
        "url_validator"
    }
}

// 注册（通常在启动时）
event_bus().register::<LinkCreatingEvent, _>(
    Priority::Highest, // 校验要最先执行
    UrlValidatorHandler,
);
```

### 监控处理器（Monitor 优先级）

```rust
pub struct AuditLogHandler;

#[async_trait]
impl EventHandler<LinkCreatingEvent> for AuditLogHandler {
    async fn handle(&self, ctx: &mut EventContext<LinkCreatingEvent>) {
        // Monitor 优先级的处理器可以看到事件是否被取消
        if ctx.is_cancelled() {
            info!(
                code = %ctx.event().code,
                reason = ?ctx.cancel_reason(),
                "Link creation was cancelled"
            );
        } else {
            info!(
                code = %ctx.event().code,
                url = %ctx.event().target_url,
                "Link creation approved"
            );
        }
    }

    fn name(&self) -> &'static str {
        "audit_log"
    }
}

// Monitor 优先级：最后执行，不能取消
event_bus().register::<LinkCreatingEvent, _>(
    Priority::Monitor,
    AuditLogHandler,
);
```

### 插件自定义事件

```rust
// 插件定义自己的事件类型
#[derive(Debug, Clone)]
pub struct MyPluginEvent {
    pub data: String,
}

impl Event for MyPluginEvent {
    fn name(&self) -> &'static str { "myplugin.custom_event" }
    fn cancellable(&self) -> bool { true }
}

// 插件发布事件
event_bus().fire(MyPluginEvent {
    data: "hello".to_string(),
});

// 其他插件可以订阅
event_bus().register::<MyPluginEvent, _>(Priority::Normal, MyPluginHandler);
```

## 事件命名约定

事件名称使用点分隔的层级结构：

```
{domain}.{action}[.{detail}]
```

| 域名 | 示例 |
|------|------|
| `system` | `system.startup`, `system.shutdown`, `system.reload.started` |
| `link` | `link.creating`, `link.created`, `link.deleted` |
| `config` | `config.changed` |
| `plugin.{id}` | `plugin.analytics.report_generated` |

## 模块结构

```
src/system/events/
├── mod.rs              # 模块导出
├── traits.rs           # Event, EventHandler traits
├── context.rs          # EventContext
├── priority.rs         # Priority enum
├── bus.rs              # EventBus 实现
├── global.rs           # 全局实例
├── builtin/            # 内置事件定义
│   ├── mod.rs
│   ├── lifecycle.rs    # StartupEvent, ShutdownEvent
│   ├── link.rs         # Link* 事件
│   ├── reload.rs       # Reload* 事件
│   └── config.rs       # ConfigChangedEvent
└── handlers/           # 内置处理器
    ├── mod.rs
    └── logging.rs      # LoggingHandler
```

## 性能考虑

1. **避免在热路径发布事件**：重定向处理不应触发事件
2. **事件数据要小**：只包含必要信息，大数据用 Arc 包装
3. **处理器要快**：耗时操作应 spawn 新任务
4. **Monitor 处理器不阻塞**：即使日志写入慢也不影响主流程

## 与 ReloadCoordinator 的关系

`ReloadCoordinator` 保持独立，但会发布事件到 `EventBus`：

```rust
impl DefaultReloadCoordinator {
    async fn reload(&self, target: ReloadTarget) -> Result<ReloadResult> {
        // 发布开始事件
        event_bus().fire(ReloadStartedEvent { target });

        // 执行重载...
        let result = self.do_reload(target).await;

        // 发布结果事件
        match &result {
            Ok(r) => event_bus().fire(ReloadCompletedEvent {
                target,
                duration_ms: r.duration_ms,
            }),
            Err(e) => event_bus().fire(ReloadFailedEvent {
                target,
                error: e.to_string(),
            }),
        }

        result
    }
}
```

## FAQ

### Q: 为什么用 trait 而不是 enum？

A: enum 是封闭的，插件无法添加新类型。trait 允许任何类型实现 `Event`，支持插件自定义事件。

### Q: 为什么区分 `*ing` 和 `*ed` 事件？

A: `*ing` 事件在操作前触发，可取消；`*ed` 事件在操作后触发，仅用于通知。这是常见的前置/后置 hook 模式。

### Q: Monitor 优先级有什么用？

A: 用于审计日志等只读场景。它总是最后执行，能看到事件的最终状态（是否被取消），但不能修改事件。

### Q: 事件处理器抛异常怎么办？

A: 处理器的 panic 会被捕获并记录日志，不会影响其他处理器执行。建议处理器内部做好错误处理。
