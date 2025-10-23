# shortlinker

<div align="center">

[![GitHub 最新发布](https://img.shields.io/github/v/release/AptS-1547/shortlinker)](https://github.com/AptS-1547/shortlinker/releases)
[![Rust 构建状态](https://img.shields.io/github/actions/workflow/status/AptS-1547/shortlinker/rust-release.yml?label=rust%20release)](https://github.com/AptS-1547/shortlinker/actions/workflows/rust-release.yml)
[![Docker 构建状态](https://img.shields.io/github/actions/workflow/status/AptS-1547/shortlinker/docker-image.yml?label=docker%20build)](https://github.com/AptS-1547/shortlinker/actions/workflows/docker-image.yml)
[![CodeFactor 评分](https://www.codefactor.io/repository/github/apts-1547/shortlinker/badge)](https://www.codefactor.io/repository/github/apts-1547/shortlinker)
[![MIT 协议](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker 拉取数](https://img.shields.io/docker/pulls/e1saps/shortlinker)](https://hub.docker.com/r/e1saps/shortlinker)

**一款极简主义的 URL 缩短服务，支持 HTTP 307 重定向，使用 Rust 构建，易于部署，极速响应。**

[English](README.md) • [中文](README.zh.md)

![管理面板界面](assets/admin-panel-dashboard.png)

</div>

## 🚀 性能基准（v0.2.0）

**测试环境**

- 操作系统：Linux
- CPU：12代 Intel Core i5-12500，单核
- 压测工具：[`wrk`](https://github.com/wg/wrk)

| 类型       | 场景                  | QPS 峰值         | 缓存命中 | 布隆过滤器 | 数据库访问 |
|------------|-----------------------|------------------|-----------|--------------|--------------|
| 命中缓存   | 热门链接（重复访问） | **696,962.45**   | ✅ 是     | ✅ 是         | ❌ 否        |
| 未命中缓存 | 冷门链接（随机访问） | **600,622.46**   | ❌ 否     | ✅ 是         | ✅ 是        |

> 💡 即使在缓存未命中时，系统仍能维持近 60 万 QPS，展示了 SQLite + actix-web + 异步缓存的卓越性能。

---

## ✨ 功能亮点

- 🚀 **高性能**：Rust + actix-web 构建
- 🔧 **运行时动态管理**：添加/删除链接无需重启服务
- 🎲 **智能短码生成**：支持自定义和随机短码
- ⏰ **支持过期时间**：灵活设置链接有效期（v0.1.1+）
- 💾 **多种存储后端**：SQLite、JSON 文件
- 🖥️ **跨平台支持**：Linux、Windows、macOS
- 🛡️ **管理 API**：支持 Bearer Token 的 HTTP API（v0.0.5+）
- 💉 **健康检查 API**：服务存活与就绪检查接口
- 🐳 **Docker 镜像**：适配容器部署，体积小巧
- 🎨 **美观 CLI**：带有颜色高亮的命令行工具
- 🔌 **Unix Socket 支持**

---

## 🚀 快速开始

### 本地运行

```bash
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker
cargo run
````

### Docker 部署

```bash
# TCP 端口模式
docker run -d -p 8080:8080 -v $(pwd)/data:/data e1saps/shortlinker

# Unix Socket 模式
docker run -d -v $(pwd)/data:/data -v $(pwd)/sock:/sock \
  -e UNIX_SOCKET=/sock/shortlinker.sock e1saps/shortlinker
```

---

## 🧪 使用示例

域名绑定后（如 `https://esap.cc`）：

* `https://esap.cc/github` → 自定义短链接
* `https://esap.cc/aB3dF1` → 随机短链接
* `https://esap.cc/` → 默认首页跳转

---

## 🔧 命令行管理示例

```bash
# 启动服务
./shortlinker

# 添加链接
./shortlinker add github https://github.com             # 自定义短码
./shortlinker add https://github.com                    # 随机短码
./shortlinker add github https://new-url.com --force    # 覆盖已有短码

# 设置相对时间（v0.1.1+）
./shortlinker add daily https://example.com --expire 1d
./shortlinker add weekly https://example.com --expire 1w
./shortlinker add complex https://example.com --expire 1d2h30m

# 管理操作
./shortlinker update github https://new-github.com --expire 30d
./shortlinker list
./shortlinker remove github

# 服务控制
./shortlinker start
./shortlinker stop
./shortlinker restart
```

---

## 🔐 管理 API（v0.0.5+）

启用管理功能：

```bash
export ADMIN_TOKEN=你的管理密钥
export ADMIN_ROUTE_PREFIX=/admin  # 可选前缀
```

### API 示例

```bash
# 获取所有链接
curl -H "Authorization: Bearer 你的管理密钥" http://localhost:8080/admin/link

# 创建链接
curl -X POST \
     -H "Authorization: Bearer 你的管理密钥" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com","expires_at":"7d"}' \
     http://localhost:8080/admin/link
```

---

## ❤️ 健康检查

```bash
export HEALTH_TOKEN=你的健康密钥

# 总体健康检查
curl -H "Authorization: Bearer $HEALTH_TOKEN" http://localhost:8080/health

# 就绪检查
curl http://localhost:8080/health/ready

# 存活检查
curl http://localhost:8080/health/live
```

---

## 🕒 时间格式支持

### 相对时间（推荐）

```bash
1s, 5m, 2h, 1d, 1w, 1M, 1y
1d2h30m  # 组合时间格式
```

### 绝对时间（RFC3339）

```bash
2024-12-31T23:59:59Z
2024-12-31T23:59:59+08:00
```

---

## ⚙️ 配置方式

**shortlinker 现在支持 TOML 配置文件！**

支持 TOML 配置文件和环境变量两种方式，TOML 配置更清晰易读，推荐使用。

### 自定义配置文件路径

可以使用 `-c` 或 `--config` 参数指定自定义配置文件路径：

```bash
# 使用自定义配置文件
./shortlinker -c /path/to/your/config.toml
./shortlinker --config /path/to/your/config.toml

# 如果指定的文件不存在，会自动创建默认配置
./shortlinker -c /etc/shortlinker/custom.toml
# [INFO] Configuration file not found: /etc/shortlinker/custom.toml
# [INFO] Creating default configuration file...
# [INFO] Default configuration file created at: /etc/shortlinker/custom.toml
```

### TOML 配置文件

创建 `config.toml` 文件：

```toml
[server]
# 服务器监听地址
host = "127.0.0.1"
# 服务器监听端口
port = 8080
# Unix Socket 路径（如果设置了，会覆盖 host 和 port）
# unix_socket = "/tmp/shortlinker.sock"
# CPU 核心数量（默认为系统核心数）
cpu_count = 4

[storage]
# 存储后端类型：sqlite, postgres, mysql, mariadb
# 💡 此字段现在是可选的 - 数据库类型可以从 DATABASE_URL 自动推断
# 如果指定，将覆盖自动检测
type = "sqlite"
# 数据库连接 URL 或文件路径
# 数据库类型会从 URL scheme 自动检测：
# - sqlite:// 或 .db/.sqlite 文件 → SQLite
# - postgres:// 或 postgresql:// → PostgreSQL
# - mysql:// → MySQL
# - mariadb:// → MariaDB（使用 MySQL 协议）
database_url = "shortlinks.db"
# 数据库连接池大小
pool_size = 10
# 数据库连接超时（秒）
timeout = 30

[cache]
# 缓存类型：memory, redis（目前仅支持 memory)
type = "memory"
# 默认缓存过期时间（秒）
default_ttl = 3600

[cache.redis]
# Redis 连接 URL
url = "redis://127.0.0.1:6379/"
# Redis 键前缀
key_prefix = "shortlinker:"
# Redis 连接池大小
pool_size = 10

[cache.memory]
# 内存缓存最大容量（条目数）
max_capacity = 10000

[api]
# 管理 API Token（留空禁用管理 API）
admin_token = ""
# 健康检查 API Token（留空则使用 admin_token）
health_token = ""

[routes]
# 管理 API 路由前缀
admin_prefix = "/admin"
# 健康检查路由前缀
health_prefix = "/health"
# 前端面板路由前缀
frontend_prefix = "/panel"

[features]
# 是否启用 Web 管理面板
enable_admin_panel = false
# 随机短码长度
random_code_length = 6
# 默认跳转 URL
default_url = "https://esap.cc/repo"

[logging]
# 日志等级：trace, debug, info, warn, error
level = "info"
```

**配置文件加载规则：**

使用 `-c/--config` 参数时：
- 使用指定的路径（不存在则自动创建）
- 示例：`./shortlinker -c /path/to/config.toml`

不使用参数时：
- 只在当前目录查找 `config.toml`
- 找不到则使用内存中的默认配置

### 环境变量（向后兼容）

仍然支持原有的环境变量配置方式，**环境变量会覆盖 TOML 配置**：

| 变量                      | 默认值                     | 说明                                        |
| ----------------------- | ------------------------ | ------------------------------------------- |
| `SERVER_HOST`           | `127.0.0.1`             | 监听地址                                      |
| `SERVER_PORT`           | `8080`                  | 监听端口                                      |
| `UNIX_SOCKET`           | *(empty)*               | Unix Socket 路径（会覆盖 HOST/PORT）            |
| `CPU_COUNT`             | *(auto)*                | 工作线程数（默认为 CPU 核心数）                      |
| `DATABASE_BACKEND`      | *(auto-detect)*         | 存储类型：sqlite, postgres, mysql, mariadb。**可选**：不设置则从 DATABASE_URL 自动检测 |
| `DATABASE_URL`          | `shortlinks.db`         | 数据库 URL 或文件路径。**支持自动检测** URL scheme    |
| `DATABASE_POOL_SIZE`    | `10`                    | 数据库连接池大小                                 |
| `DATABASE_TIMEOUT`      | `30`                    | 数据库连接超时（秒）                              |
| `CACHE_TYPE`            | `memory`                | 缓存类型：memory, redis                       |
| `CACHE_DEFAULT_TTL`     | `3600`                  | 默认缓存过期时间（秒）                             |
| `REDIS_URL`             | `redis://127.0.0.1:6379/` | Redis 连接地址                             |
| `REDIS_KEY_PREFIX`      | `shortlinker:`          | Redis 键前缀                                 |
| `REDIS_POOL_SIZE`       | `10`                    | Redis 连接池大小                              |
| `MEMORY_MAX_CAPACITY`   | `10000`                 | 内存缓存最大容量（条目数）                          |
| `ADMIN_TOKEN`           | *(empty)*               | 管理 API 密钥                                |
| `HEALTH_TOKEN`          | *(empty)*               | 健康检查密钥                                   |
| `ADMIN_ROUTE_PREFIX`    | `/admin`                | 管理 API 路由前缀                             |
| `HEALTH_ROUTE_PREFIX`   | `/health`               | 健康检查路由前缀                                |
| `FRONTEND_ROUTE_PREFIX` | `/panel`                | Web 管理面板路由前缀                            |
| `ENABLE_ADMIN_PANEL`    | `false`                 | 启用 Web 管理面板                             |
| `RANDOM_CODE_LENGTH`    | `6`                     | 随机短码长度                                   |
| `DEFAULT_URL`           | `https://esap.cc/repo`  | 默认跳转 URL                                 |
| `RUST_LOG`              | `info`                  | 日志等级                                     |

---

## 📦 存储后端

Shortlinker 现在使用 **Sea-ORM** 进行数据库操作，提供：
- ✅ **原子化 upsert 操作**（防止竞态条件）
- ✅ **从 DATABASE_URL 自动检测数据库类型**（无需指定 DATABASE_BACKEND）
- ✅ **自动创建 SQLite 数据库文件**（如果不存在）
- ✅ **自动执行数据库模式迁移**

### 支持的数据库

- **SQLite**（默认）：生产就绪，推荐用于单节点部署
- **MySQL / MariaDB**：生产就绪，推荐用于多节点部署
- **PostgreSQL**：生产就绪，推荐用于企业级部署

### 数据库 URL 示例

```bash
# SQLite - 自动检测
DATABASE_URL=links.db                    # 相对路径
DATABASE_URL=/var/lib/shortlinker/links.db  # 绝对路径
DATABASE_URL=sqlite://data/links.db      # 显式 SQLite URL

# PostgreSQL - 自动检测
DATABASE_URL=postgres://user:pass@localhost:5432/shortlinker
DATABASE_URL=postgresql://user:pass@host:5432/db?sslmode=require

# MySQL - 自动检测
DATABASE_URL=mysql://user:pass@localhost:3306/shortlinker
DATABASE_URL=mysql://user:pass@host:3306/db?charset=utf8mb4

# MariaDB - 自动检测（使用 MySQL 协议）
DATABASE_URL=mariadb://user:pass@localhost:3306/shortlinker
```

> 💡 **提示**：`DATABASE_BACKEND` 环境变量现在是**可选的**。数据库类型会从 `DATABASE_URL` 自动推断。只有在需要覆盖自动检测时才需要指定。

---

## 📡 部署示例

### Nginx 反向代理

```nginx
server {
    listen 80;
    server_name esap.cc;
    location / {
        proxy_pass http://127.0.0.1:8080;
    }
}
```

### systemd 服务

```ini
[Unit]
Description=ShortLinker 服务
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/shortlinker
ExecStart=/opt/shortlinker/shortlinker
Restart=always
Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080

[Install]
WantedBy=multi-user.target
```

---

## 🔧 开发者指南

```bash
cargo run           # 开发运行
cargo build --release  # 生产构建
cargo test          # 运行测试
cargo fmt && cargo clippy  # 格式化与静态检查
```

---

## 🧩 相关模块

* Web 管理面板：`admin-panel/`
* Cloudflare Worker：无服务器版，位于 `cf-worker/`

---

## 📜 协议

MIT License © AptS:1547

<pre>
        ／＞　 フ
       | 　_　_|    AptS:1547
     ／` ミ＿xノ    — shortlinker assistant bot —
    /　　　　 |
   /　 ヽ　　 ﾉ      Rust / SQLite / Bloom / CLI
   │　　|　|　|
／￣|　　 |　|　|
(￣ヽ＿_ヽ_)__)
＼二)

   「ready to 307 !」
</pre>

> [🔗 Visit Project Docs](https://esap.cc/docs)
> [💬 Powered by AptS:1547](https://github.com/AptS-1547)
