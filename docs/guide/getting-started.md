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
```

## 第二步：启动服务

```bash
# 启动服务器
./shortlinker

# 看到以下输出表示成功：
# [INFO] Starting server at http://127.0.0.1:8080
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

### 查看所有短链接
```bash
./shortlinker list
```

### 删除短链接
```bash
./shortlinker remove github
```

### 添加临时链接
```bash
./shortlinker add temp https://example.com --expire 2024-12-31T23:59:59Z
```

### 强制覆盖
```bash
./shortlinker add github https://github.com --force
```

## 服务管理

### 停止服务
```bash
# 方式1：Ctrl+C
# 方式2：发送信号
kill $(cat shortlinker.pid)
```

### 重载配置
```bash
# Unix 系统
kill -HUP $(cat shortlinker.pid)
```

## 生产环境建议

### 反向代理
建议使用 Nginx 或 Caddy 作为反向代理：

```nginx
# Nginx 配置示例
server {
    listen 80;
    server_name your-domain.com;
    location / {
        proxy_pass http://127.0.0.1:8080;
    }
}
```

### 系统服务
使用 systemd 管理服务：

```bash
# 安装为系统服务
sudo cp shortlinker.service /etc/systemd/system/
sudo systemctl enable shortlinker
sudo systemctl start shortlinker
```

## 下一步

恭喜！您已经成功配置了 Shortlinker。接下来可以：

- 📋 学习 [CLI 命令详情](/cli/commands)
- 🚀 查看 [部署指南](/deployment/) 进行生产部署
- ⚙️ 了解 [高级配置](/config/examples)
