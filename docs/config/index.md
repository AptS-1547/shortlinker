# 环境变量配置

Shortlinker 通过环境变量进行配置，支持 `.env` 文件和系统环境变量。

## 配置方式

### .env 文件（推荐）
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

## 配置参数

### 服务器配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `SERVER_HOST` | String | `127.0.0.1` | 监听地址 |
| `SERVER_PORT` | Integer | `8080` | 监听端口 |
| `UNIX_SOCKET` | String | *(空)* | Unix 套接字路径（设置后忽略 HOST/PORT） |
| `CPU_COUNT` | Integer | *(自动)* | 工作线程数量（默认为CPU核心数） |
| `DEFAULT_URL` | String | `https://esap.cc/repo` | 根路径重定向地址 |
| `RANDOM_CODE_LENGTH` | Integer | `6` | 随机短码长度 |

### 存储配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `STORAGE_BACKEND` | String | `sqlite` | 存储类型：`sqlite`、`file`、`sled` |
| `DB_FILE_NAME` | String | `links.db` | 数据库文件路径 |

> 详细的存储后端配置请参考 [存储后端](/config/storage)

### API 配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `ADMIN_TOKEN` | String | *(空)* | Admin API 鉴权令牌，**为空时禁用** |
| `ADMIN_ROUTE_PREFIX` | String | `/admin` | Admin API 路由前缀 |
| `HEALTH_TOKEN` | String | *(空)* | 健康检查 API 鉴权令牌，**为空时禁用** |
| `HEALTH_ROUTE_PREFIX` | String | `/health` | 健康检查 API 路由前缀 |
| `ENABLE_ADMIN_PANEL` | Boolean | `false` | 启用 Web 管理界面（需先构建且需同时设置 ADMIN_TOKEN） |
| `FRONTEND_ROUTE_PREFIX` | String | `/panel` | Web 管理界面路由前缀 |
> **注意**：Web 管理界面是新推出的特性，可能仍在完善中。

> 详细的 API 配置请参考 [Admin API](/api/admin) 和 [健康检查 API](/api/health)

### 日志配置

| 参数 | 类型 | 默认值 | 可选值 |
|------|------|--------|-------|
| `RUST_LOG` | String | `info` | `error`, `warn`, `info`, `debug`, `trace` |

## 配置示例

### 开发环境
```bash
# 基础配置
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=debug

# 存储配置 - 文件存储便于调试
STORAGE_BACKEND=file
DB_FILE_NAME=dev-links.json

# API 配置 - 开发环境使用简单token
ADMIN_TOKEN=dev_admin
HEALTH_TOKEN=dev_health
```

### 生产环境
```bash
# 基础配置
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
CPU_COUNT=8
RUST_LOG=info
DEFAULT_URL=https://your-domain.com

# 存储配置 - SQLite 生产级性能
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db

# API 配置 - 使用强密码
ADMIN_TOKEN=very_secure_production_token_456
HEALTH_TOKEN=very_secure_health_token_789
```

### Docker 环境
```bash
# 服务器配置 - TCP
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
CPU_COUNT=4

# 服务器配置 - Unix 套接字
# UNIX_SOCKET=/tmp/shortlinker.sock

# 存储配置
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db

# API 配置
ADMIN_TOKEN=docker_admin_token_123
HEALTH_TOKEN=docker_health_token_456
```

### 最小配置（仅重定向功能）
```bash
# 只提供重定向服务，不启用管理功能
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
# 不设置 ADMIN_TOKEN 和 HEALTH_TOKEN
```

## API 访问控制

| 场景 | ADMIN_TOKEN | HEALTH_TOKEN | 说明 |
|------|-------------|--------------|------|
| **仅运行服务** | 不设置 | 不设置 | 最安全，仅提供重定向功能 |
| **运行+管理** | 设置 | 不设置 | 启用管理功能 |
| **运行+监控** | 不设置 | 设置 | 启用监控功能 |
| **完整功能** | 设置 | 设置 | 启用所有功能 |

## 配置优先级

1. **命令行环境变量**（最高）
2. **系统环境变量**
3. **`.env` 文件**
4. **程序默认值**（最低）

## 配置验证

启动时会显示当前配置状态：

```bash
[INFO] Starting server at http://127.0.0.1:8080
[INFO] SQLite storage initialized with 0 links
[INFO] Admin API available at: /admin
[INFO] Health API available at: /health
```

## 配置更新

### 支持热重载
- ✅ 存储文件内容变更
- ❌ 服务器地址和端口（需重启）
- ❌ API 配置（需重启）

### 重载方法
```bash
# Unix 系统
kill -USR1 $(cat shortlinker.pid)

# Windows 系统  
echo "" > shortlinker.reload
```

## 下一步

- 📋 查看 [存储后端配置](/config/storage) 了解详细存储选项
- 🚀 学习 [部署配置](/deployment/) 生产环境设置
- 🛡️ 了解 [Admin API](/api/admin) 管理接口使用
- 🏥 了解 [健康检查 API](/api/health) 监控接口使用
