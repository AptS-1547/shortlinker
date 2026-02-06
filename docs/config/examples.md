# 配置示例与热重载

本页包含开发/生产/Docker 配置示例，以及热重载能力说明。

## 配置示例

### 开发环境

```toml
# config.toml（开发）
[server]
host = "127.0.0.1"
port = 8080

[database]
database_url = "dev-links.db"

[logging]
level = "debug"
```

> 运行时配置（如 `features.enable_admin_panel`、`api.health_token`）请通过 Admin API 或 CLI 写入数据库；`api.admin_token` 请使用 `./shortlinker reset-password` 重置。

### 生产环境

```toml
# config.toml
[server]
host = "127.0.0.1"
port = 8080
cpu_count = 8

[database]
database_url = "/data/shortlinks.db"
pool_size = 20

[cache]
type = "memory"
default_ttl = 7200

[cache.memory]
max_capacity = 50000

[logging]
level = "info"
format = "json"
file = "/var/log/shortlinker/app.log"
enable_rotation = true
```

### Docker 环境

Docker 场景建议通过**挂载配置文件**来设置启动配置（尤其是把 `server.host` 设为 `0.0.0.0`）：

```toml
# /config.toml（容器内）
[server]
host = "0.0.0.0"
port = 8080

[database]
database_url = "sqlite:///data/links.db"
```

运行时配置（写入数据库）可在容器内用 CLI 设置；其中标记为“需要重启”的配置需要重启容器生效：

```bash
# 启用管理面板（需要重启）
/shortlinker config set features.enable_admin_panel true

# 配置 Health Bearer Token（无需重启）
/shortlinker config set api.health_token "your_health_token"
```

## 热重载

Shortlinker 的“热重载/热生效”主要分两类：

1. **短链接数据热重载**：让服务重新从存储加载短链接并重建缓存（适用于 CLI/TUI 直接写数据库后通知服务刷新缓存）。
2. **运行时配置热生效**：通过 Admin API 更新“无需重启”的配置时，会直接同步到内存配置并立即生效。

### 支持热生效/热重载的内容

- ✅ 短链接数据（缓存重建）
- ✅ 标记为“无需重启”的运行时配置（通过 Admin API 更新时立即生效）
- ✅ Cookie 配置（`api.cookie_*`）：对新下发的 Cookie 生效，修改后建议重新登录获取新 Cookie

### 不支持热重载的配置

- ❌ 服务器地址和端口
- ❌ 数据库连接
- ❌ 缓存类型
- ❌ 路由前缀

### 重载方法

```bash
# 1) 重载短链接数据/缓存（Unix 系统 - 发送 SIGUSR1 信号）
# 注意：SIGUSR1 只会触发短链接数据/缓存重载，不会重载运行时配置
kill -USR1 $(cat shortlinker.pid)

# 2) 重载运行时配置（通过 Admin API）
# 说明：如果你是通过 Admin API 直接更新配置（PUT /admin/v1/config/{key}），
#       且该配置“无需重启”，一般不需要额外 reload。
#       如果你是直接改数据库（例如使用 `./shortlinker config set`），可以调用该接口让服务重新从 DB 加载配置。
#
# 先登录获取 cookies（如已存在 cookies.txt 可跳过）
curl -sS -X POST \
     -H "Content-Type: application/json" \
     -c cookies.txt \
     -d '{"password":"your_admin_token"}' \
     http://localhost:8080/admin/v1/auth/login

CSRF_TOKEN=$(awk '$6=="csrf_token"{print $7}' cookies.txt | tail -n 1)

curl -X POST \
     -b cookies.txt \
     -H "X-CSRF-Token: ${CSRF_TOKEN}" \
     http://localhost:8080/admin/v1/config/reload
```

