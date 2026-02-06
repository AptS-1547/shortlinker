# 启动配置参数

这些配置存储在 `config.toml` 中，修改后需要重启服务。

> 提示：如需配置数据库 URL 细节与不同后端差异，见 [存储后端配置](/config/storage)。

### 服务器配置

| TOML 键 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `server.host` | String | `127.0.0.1` | 监听地址（Docker 中通常用 `0.0.0.0`） |
| `server.port` | Integer | `8080` | 监听端口 |
| `server.unix_socket` | String | *(空)* | Unix 套接字路径（设置后忽略 `server.host`/`server.port`） |
| `server.cpu_count` | Integer | *(自动)* | Worker 数量（默认 CPU 核心数，最大 32） |

### 数据库配置

| TOML 键 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `database.database_url` | String | `shortlinks.db` | 数据库连接 URL 或文件路径（后端会自动从该值推断数据库类型） |
| `database.pool_size` | Integer | `10` | 连接池大小（仅 MySQL/PostgreSQL 生效；SQLite 使用内置池配置） |
| `database.timeout` | Integer | `30` | *(当前版本暂未使用；连接超时固定为 8s)* |
| `database.retry_count` | Integer | `3` | 部分数据库操作的重试次数 |
| `database.retry_base_delay_ms` | Integer | `100` | 重试基础延迟（毫秒） |
| `database.retry_max_delay_ms` | Integer | `2000` | 重试最大延迟（毫秒） |

> 详细的存储后端配置请参考 [存储后端](/config/storage)

### 缓存配置

| TOML 键 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `cache.type` | String | `memory` | 缓存类型：`memory` / `redis` |
| `cache.default_ttl` | Integer | `3600` | 默认缓存过期时间（秒） |
| `cache.redis.url` | String | `redis://127.0.0.1:6379/` | Redis 连接地址 |
| `cache.redis.key_prefix` | String | `shortlinker:` | Redis 键前缀 |
| `cache.memory.max_capacity` | Integer | `10000` | 内存缓存最大容量 |

### 日志配置

| TOML 键 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `logging.level` | String | `info` | 日志等级：error / warn / info / debug / trace |
| `logging.format` | String | `text` | 输出格式：`text` / `json` |
| `logging.file` | String | *(空)* | 日志文件路径（为空则输出到 stdout） |
| `logging.max_backups` | Integer | `5` | 日志轮转保留文件数 |
| `logging.enable_rotation` | Boolean | `true` | 是否启用轮转（当前为按天轮转） |
| `logging.max_size` | Integer | `100` | *(当前版本暂未使用；轮转按天而非按大小)* |

> 日志格式与文件输出通过 `config.toml` 的 `[logging]` 配置设置（例如 `logging.format`、`logging.file`）。

### IPC 配置

| TOML 键 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `ipc.enabled` | Boolean | `true` | 是否启用 IPC 服务端（CLI/TUI 与运行中服务通信依赖它） |
| `ipc.socket_path` | String | *(平台默认)* | 自定义 IPC 路径（Unix socket / Windows named pipe） |
| `ipc.max_message_size` | Integer | `65536` | IPC 消息最大字节数 |
| `ipc.timeout` | Integer | `5` | 常规 IPC 操作超时（秒） |
| `ipc.reload_timeout` | Integer | `30` | 配置/数据重载类 IPC 超时（秒） |
| `ipc.bulk_timeout` | Integer | `60` | 批量导入导出 IPC 超时（秒） |

> 说明：
> - 路径优先级：CLI `--socket` > `ipc.socket_path` > 平台默认值。默认值为 Unix `./shortlinker.sock`，Windows `\\.\\pipe\\shortlinker`。
> - Unix 下 IPC socket 文件权限固定为 `0600`（仅属主读写）。
> - 若 `ipc.enabled=false`，`./shortlinker status` 与 CLI/TUI 的 IPC 同步能力不可用；运行时配置需通过 Admin API `POST /admin/v1/config/reload` 或重启生效。

### GeoIP（分析）配置

| TOML 键 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `analytics.maxminddb_path` | String | *(空)* | MaxMindDB 文件路径（GeoLite2-City.mmdb，可选；可读时优先使用本地解析） |
| `analytics.geoip_api_url` | String | `http://ip-api.com/json/{ip}?fields=status,countryCode,city` | 外部 GeoIP API URL（MaxMindDB 不可用时 fallback；`{ip}` 为占位符） |

> 说明：
> - Provider 选择：`analytics.maxminddb_path` 可读时使用本地 MaxMind；否则使用外部 API（`analytics.geoip_api_url`）。
> - 外部 API Provider 内置缓存（不可配置）：LRU 最大 10000 条，TTL 15 分钟（包含失败的负缓存）；同一 IP 的并发查询会合并为一次请求；单次请求超时 2 秒。
