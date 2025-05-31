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
| `ADMIN_TOKEN` | String | *(空字符串)* | Admin API 鉴权令牌，**为空时禁用 Admin API** |
| `ADMIN_ROUTE_PREFIX` | String | `/admin` | Admin API 路由前缀 |

**重要说明**：
- 默认情况下 Admin API 是**禁用**的，以确保安全性
- 只有设置了 `ADMIN_TOKEN` 环境变量后，Admin API 才会启用
- 未设置 token 时访问 Admin 路由将返回 404 Not Found

### 存储配置 (v0.1.0+)

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `STORAGE_BACKEND` | String | `sqlite` | 存储后端类型（`sqlite`、`file` 或 `sled`），v0.1.0+ 支持多后端 |
| `DB_FILE_NAME` | String | `links.db`（SQLite），`links.json`（文件），`links.sled`（Sled） | 数据库文件路径（根据后端而定） |

**版本说明**：
- **v0.1.0+**: 支持多种存储后端，SQLite 为默认选择
- **< v0.1.0**: 仅支持文件存储，无需配置 `STORAGE_BACKEND`

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
[INFO] Admin API is disabled (ADMIN_TOKEN not set)
# 或者
[INFO] Admin API available at: /admin/link
```

## 常用配置场景

### 开发环境
```bash
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=debug
RANDOM_CODE_LENGTH=4

# 存储配置 - 开发环境可选择文件存储便于调试
STORAGE_BACKEND=file
DB_FILE_NAME=dev-links.json

# 启用 Admin API（开发环境）
ADMIN_TOKEN=dev_token_123
```

### 生产环境
```bash
SERVER_HOST=127.0.0.1  # 通过反向代理访问
SERVER_PORT=8080
RUST_LOG=info
RANDOM_CODE_LENGTH=8

# 存储配置 - 生产环境推荐 SQLite（v0.1.0+）
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db

# 生产环境强烈建议设置强密码
ADMIN_TOKEN=very_secure_production_token_456
```

### Docker 环境
```bash
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# SQLite 存储（推荐，v0.1.0+）
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db

# 或者文件存储（兼容旧版本）
# STORAGE_BACKEND=file
# DB_FILE_NAME=/data/links.json

# 可选：启用 Admin API
ADMIN_TOKEN=docker_admin_token_789
```

### 版本兼容配置

#### v0.1.0+ 配置
```bash
# 明确指定存储类型（推荐）
STORAGE_BACKEND=sqlite
DB_FILE_NAME=data/links.db
```

#### v0.0.x 兼容配置
```bash
# 旧版本升级时，继续使用文件存储
STORAGE_BACKEND=file
DB_FILE_NAME=links.json
```

## 配置更新

### 支持热重载
- ✅ 存储文件内容变更
- ❌ 服务器地址和端口
- ❌ 日志级别
- ❌ Admin API 配置（需要重启服务器）

### 重载方法
```bash
# Unix 系统发送 SIGHUP 信号
kill -HUP $(cat shortlinker.pid)

# Windows 系统自动监控文件变化
```

## 下一步

- 📋 查看 [配置示例](/config/examples) 了解不同场景配置
- 🚀 学习 [部署配置](/deployment/) 生产环境设置
- 🛡️ 了解 [Admin API](/api/admin) 管理接口使用方法
