# 快速开始

本指南帮助您在 5 分钟内完成 Shortlinker 的配置和基本使用。

## 前置条件

请先完成 [安装指南](/guide/installation) 中的任一安装方式。

## 第一步：基础配置

创建配置文件 `.env`：

```bash
# 最小配置
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
DEFAULT_URL=https://example.com

# 可选：启用管理和监控功能
# ADMIN_TOKEN=your_admin_token
# HEALTH_TOKEN=your_health_token
```

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

# 重载配置（Unix 系统）
kill -USR1 $(cat shortlinker.pid)
```

## 生产环境快速配置

### 推荐配置
```bash
# 生产环境 .env 配置
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db
DEFAULT_URL=https://your-domain.com

# 启用 API 功能
ADMIN_TOKEN=your_secure_admin_token
HEALTH_TOKEN=your_secure_health_token
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
      - ./data:/data
    environment:
      - STORAGE_BACKEND=sqlite
      - DB_FILE_NAME=/data/links.db
```

## 下一步

恭喜！您已经成功配置了 Shortlinker。接下来可以：

- 📋 学习 [CLI 命令详情](/cli/commands) - 掌握所有命令选项
- 🚀 查看 [部署指南](/deployment/) - 进行生产环境部署
- ⚙️ 了解 [配置选项](/config/) - 自定义高级配置
- 🛡️ 使用 [Admin API](/api/admin) - HTTP 接口管理
- 🏥 配置 [健康检查](/api/health) - 服务监控
