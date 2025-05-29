# 环境变量配置

Shortlinker 通过环境变量进行配置，支持 `.env` 文件和系统环境变量。

## 配置方式

### .env 文件（推荐）
在项目根目录创建 `.env` 文件：

```bash
# .env
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
DEFAULT_URL=https://example.com
```

### 系统环境变量
```bash
export SERVER_HOST=0.0.0.0
export SERVER_PORT=8080
./shortlinker
```

### 命令行指定
```bash
SERVER_PORT=3000 ./shortlinker
```

## 配置参数

### 服务器配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `SERVER_HOST` | String | `127.0.0.1` | 监听地址 |
| `SERVER_PORT` | Integer | `8080` | 监听端口 |

### 功能配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `DEFAULT_URL` | String | `https://esap.cc/repo` | 根路径重定向地址 |
| `RANDOM_CODE_LENGTH` | Integer | `6` | 随机短码长度 |

### Admin API 配置 (v0.0.5+)

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `ADMIN_TOKEN` | String | `default_admin_token` | Admin API 鉴权令牌 |
| `ADMIN_ROUTE_PREFIX` | String | `/admin` | Admin API 路由前缀 |

### 存储配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `LINKS_FILE` | String | `links.json` | 存储文件路径 |

### 日志配置

| 参数 | 类型 | 默认值 | 可选值 |
|------|------|--------|-------|
| `RUST_LOG` | String | `info` | `error`, `warn`, `info`, `debug`, `trace` |

## 配置优先级

1. **命令行环境变量**（最高）
2. **系统环境变量**
3. **`.env` 文件**
4. **程序默认值**（最低）

## 配置验证

启动时会显示当前配置：

```bash
[INFO] Starting server at http://127.0.0.1:8080
[INFO] Storage: links.json
[INFO] Default URL: https://example.com
```

## 常用配置场景

### 开发环境
```bash
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=debug
RANDOM_CODE_LENGTH=4
```

### 生产环境
```bash
SERVER_HOST=127.0.0.1  # 通过反向代理访问
SERVER_PORT=8080
RUST_LOG=info
RANDOM_CODE_LENGTH=8
```

### Docker 环境
```bash
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
LINKS_FILE=/data/links.json
```

## 配置更新

### 支持热重载
- ✅ 存储文件内容变更
- ❌ 服务器地址和端口
- ❌ 日志级别

### 重载方法
```bash
# Unix 系统发送 SIGHUP 信号
kill -HUP $(cat shortlinker.pid)

# Windows 系统自动监控文件变化
```

## 下一步

- 📋 查看 [配置示例](/config/examples) 了解不同场景配置
- 🚀 学习 [部署配置](/deployment/) 生产环境设置
