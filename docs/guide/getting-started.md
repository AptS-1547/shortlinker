# 快速开始

本指南帮助您在 5 分钟内完成 Shortlinker 的配置和基本使用。

## 前置条件

请先完成 [安装指南](/guide/installation) 中的任一安装方式。

## 第一步：基础配置

### 方式一：使用 TOML 配置文件（推荐）

推荐使用 `config generate` 命令生成配置文件：

```bash
./shortlinker config generate config.toml
# 生成启动配置模板（server/database/cache/logging/analytics）
```

然后根据需要修改 `config.toml`：

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
database_url = "shortlinks.db"

[logging]
level = "info"
```

::: tip
如果不创建配置文件，程序会使用内置的默认配置运行。
:::

> 说明：运行时配置（例如 `features.default_url`、`api.health_token`、`features.enable_admin_panel`）存储在数据库中，需通过 Admin API 或 CLI 修改；当前版本不会从 `config.toml`/环境变量读取这类配置。

## 第二步：启动服务

```bash
# 启动服务器
./shortlinker

# 看到以下输出表示成功：
# [INFO] Starting server at http://127.0.0.1:8080
# [INFO] SQLite storage initialized with 0 links
```

## 第三步：添加短链接

```bash
# 自定义短码
./shortlinker add github https://github.com

# 随机短码
./shortlinker add https://www.google.com
# 输出：✓ 已添加短链接: aB3dF1 -> https://www.google.com
```

## 第四步：测试访问

```bash
# 测试重定向
curl -I http://localhost:8080/github
# HTTP/1.1 307 Temporary Redirect
# Location: https://github.com

# 浏览器访问
# http://localhost:8080/github
```

## 常用操作

```bash
# 查看所有短链接
./shortlinker list

# 删除短链接
./shortlinker remove github

# 添加临时链接
./shortlinker add temp https://example.com --expire 1d

# 强制覆盖
./shortlinker add github https://github.com --force
```

## 服务管理

```bash
# 停止服务
# 方式1：Ctrl+C
# 方式2：发送信号
kill $(cat shortlinker.pid)

# 查看服务状态（IPC）
./shortlinker status

# 若使用自定义 IPC 路径，可用 --socket 覆盖
./shortlinker --socket /tmp/shortlinker.sock status

# 运行时配置变更（config set/reset/import）会自动尝试通过 IPC 重载配置。
# 若 IPC 不可达（服务未运行、ipc.enabled=false、socket 路径不一致等），
# 可手动调用 Admin API `/admin/v1/config/reload` 或直接重启服务。
```

## 生产环境快速配置

### 推荐配置
```toml
# config.toml（生产）
[server]
host = "127.0.0.1"
port = 8080

[database]
database_url = "sqlite:///data/links.db"

[logging]
level = "info"
```

运行时配置（写入数据库）可通过 CLI/Admin API 设置，例如：

```bash
# 设置根路径默认跳转（无需重启）
./shortlinker config set features.default_url https://your-domain.com

# 设置 Health Bearer Token（无需重启）
./shortlinker config set api.health_token your_health_token

# 重置管理员密码（推荐）
./shortlinker reset-password
```

### 反向代理示例
```nginx
# Nginx 配置示例
server {
    listen 80;
    server_name your-domain.com;
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
    }
}
```

### Docker 快速部署
```bash
# 使用 Docker Compose
version: '3.8'
services:
  shortlinker:
    image: e1saps/shortlinker
    ports:
      - "127.0.0.1:8080:8080"
    volumes:
      - ./config.toml:/config.toml:ro
      - ./data:/data
```

## 下一步

恭喜！您已经成功配置了 Shortlinker。接下来可以：

- 📋 学习 [CLI 命令详情](/cli/commands) - 掌握所有命令选项
- 🚀 查看 [部署指南](/deployment/) - 进行生产环境部署
- ⚙️ 了解 [配置选项](/config/) - 自定义高级配置
- 🛡️ 使用 [Admin API](/api/admin) - HTTP 接口管理
- 🏥 配置 [健康检查](/api/health) - 服务监控
