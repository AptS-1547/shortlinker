# Shortlinker Cloudflare Worker

基于 Cloudflare Workers 和 KV 存储构建的无服务器短链接服务。

## 功能特性

- ⚡ **无服务器** - 运行在 Cloudflare 全球边缘网络上
- 🗄️ **KV 存储** - 使用 Cloudflare KV 进行持久化数据存储
- 🌍 **全球分发** - 全球低延迟访问
- 🔒 **安全可靠** - 内置 DDoS 防护和安全特性
- 📈 **自动扩容** - 零配置自动扩展
- 💰 **成本效益** - 按使用量付费的定价模式

## 技术栈

- **运行时**: Cloudflare Workers (Rust + WebAssembly)
- **编程语言**: Rust + `worker` crate
- **存储**: Cloudflare KV
- **构建工具**: `wasm-pack` + `wrangler`

## 开发状态

🚧 **正在开发中** - 该 Cloudflare Worker 实现目前正在开发中，将为主要的 Shortlinker 服务提供无服务器替代方案。

### 计划功能

- [ ] 短链接创建和重定向
- [ ] 基于 KV 的数据持久化
- [ ] Admin API 端点
- [ ] 速率限制和滥用防护
- [ ] 分析和使用统计
- [ ] 自定义域名支持
- [ ] 批量操作

## 项目结构

```
cf-worker/
├── src/
│   └── lib.rs              # 主要 Worker 逻辑
├── build/                  # 构建的 WebAssembly 产物
├── wrangler.toml          # Cloudflare Worker 配置
├── Cargo.toml             # Rust 依赖
└── .wrangler/             # Wrangler 缓存和状态
```

## 环境配置

### 前置要求

- [Rust](https://rustup.rs/) 工具链
- [wrangler](https://developers.cloudflare.com/workers/wrangler/) CLI
- 启用 Workers 的 Cloudflare 账户

### 安装

```bash
# 安装 wrangler
npm install -g wrangler

# 使用 Cloudflare 进行身份验证
wrangler auth

# 为 WebAssembly 安装 Rust 目标
rustup target add wasm32-unknown-unknown
```

## 配置

### KV 命名空间设置

为 Worker 创建 KV 命名空间：

```bash
# 创建生产环境 KV 命名空间
wrangler kv:namespace create "SHORTLINK_STORE"

# 创建用于开发的预览 KV 命名空间
wrangler kv:namespace create "SHORTLINK_STORE" --preview
```

### wrangler.toml 配置

```toml
# wrangler.toml
name = "shortlinker-worker"
main = "build/worker/index.js"
compatibility_date = "2024-01-01"

[build]
command = "cargo build --target wasm32-unknown-unknown --release && wasm-pack build --target no-modules --out-dir build"

[[kv_namespaces]]
binding = "SHORTLINK_STORE"
id = "your_kv_namespace_id"
preview_id = "your_preview_kv_namespace_id"

[vars]
ADMIN_TOKEN = "your_admin_token"
BASE_URL = "https://your-worker.your-subdomain.workers.dev"
```

## 开发

### 本地开发

```bash
# 进入 cf-worker 目录
cd cf-worker

# 安装依赖
cargo build

# 启动本地开发服务器
wrangler dev
```

### 生产环境构建

```bash
# 构建 WebAssembly
cargo build --target wasm32-unknown-unknown --release

# 生成 JavaScript 绑定
wasm-pack build --target no-modules --out-dir build

# 部署到 Cloudflare
wrangler deploy
```

## API 端点

Worker 将实现以下端点：

### 公共端点

- `GET /{code}` - 重定向到目标 URL
- `POST /create` - 创建新的短链接（带速率限制）

### Admin 端点（受保护）

- `GET /admin/links` - 列出所有短链接
- `POST /admin/links` - 创建新的短链接
- `GET /admin/links/{code}` - 获取特定短链接
- `PUT /admin/links/{code}` - 更新短链接
- `DELETE /admin/links/{code}` - 删除短链接

## KV 数据结构

短链接将在 KV 中按以下结构存储：

```json
{
  "code": "abc123",
  "target": "https://example.com",
  "created_at": "2024-01-01T00:00:00Z",
  "expires_at": "2024-12-31T23:59:59Z",
  "hits": 0,
  "last_accessed": "2024-01-01T00:00:00Z"
}
```

### KV 键值

- `link:{code}` - 单个短链接数据
- `stats:total` - 链接总数
- `stats:hits` - 重定向总数

## Rust 实现预览

```rust
// 未来实现结构
use worker::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ShortLink {
    code: String,
    target: String,
    created_at: String,
    expires_at: Option<String>,
    hits: u64,
}

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let router = Router::new();
    
    router
        .get_async("/:code", redirect_handler)
        .post_async("/create", create_handler)
        .get_async("/admin/links", admin_list_handler)
        .run(req, env)
        .await
}

async fn redirect_handler(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv = ctx.env.kv("SHORTLINK_STORE")?;
    // 重定向逻辑的实现
    todo!()
}
```

## 部署

### 使用 Wrangler

```bash
# 部署到生产环境
wrangler deploy

# 使用环境变量部署
wrangler deploy --var ADMIN_TOKEN:your_token
```

### 环境变量

在 Cloudflare 仪表板或通过 wrangler 配置以下变量：

- `ADMIN_TOKEN` - Admin 端点的认证令牌
- `BASE_URL` - Worker 的基础 URL
- `RATE_LIMIT` - 速率限制配置

## 监控和分析

Worker 将包含：

- 请求日志和指标
- KV 操作监控
- 错误跟踪和告警
- 使用分析仪表板

## 安全特性

- 每 IP 速率限制
- Admin 令牌认证
- 输入验证和清理
- CORS 配置
- 请求大小限制

## 性能优化

- 高效的 KV 键设计
- 缓存策略
- 最小化负载大小
- 边缘端处理

## 成本估算

Cloudflare Workers 定价（截至 2024 年）：

- **免费套餐**: 100,000 请求/天
- **付费计划**: 每月 $5，包含 1000 万请求
- **KV 存储**: 每百万次读取 $0.50，每百万次写入 $5

## 限制

- KV 最终一致性
- 每个请求 25ms CPU 时间限制
- 128MB 内存限制
- 1MB 请求/响应大小限制

## 从主服务迁移

Worker 可以作为：

1. **独立服务** - 完全替代方案
2. **边缘缓存** - 主服务的前端
3. **备份服务** - 维护期间的后备方案

## 开发路线图

该实现将分阶段开发：

1. **第一阶段**: 基本重定向功能
2. **第二阶段**: KV 存储集成
3. **第三阶段**: Admin API 实现
4. **第四阶段**: 高级功能和优化

## 相关文档

- 📖 [Shortlinker 主文档](../README.zh.md)
- 🔧 [主服务源码](../src/)
- 🎛️ [管理面板](../admin-panel/)
- ☁️ [Cloudflare Workers 文档](https://developers.cloudflare.com/workers/)

## 贡献

这是 Shortlinker 项目的一部分。请查看主项目的[贡献指南](../CONTRIBUTING.md)了解相关准则。

## 许可证

MIT License - 查看 [LICENSE](../LICENSE) 文件了解详细信息。
