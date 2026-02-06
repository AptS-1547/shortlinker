# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.5.0-beta.1] - 2026-02-06

### 🎉 Release Highlights

v0.5.0-beta.1 是一次功能增强与架构优化版本，主要亮点：

- **UTM 来源追踪** - 自动从 utm_source 参数或 Referer 推导流量来源，异步处理不阻塞重定向
- **IPC 配置化** - 新增 [ipc] 配置段，支持自定义 socket 路径、超时等参数
- **TUI 状态管理重构** - 引入 FormState 状态机，表单验证逻辑统一
- **文档结构重组** - 配置、部署、API 文档拆分为细粒度小文件

### Added
- **UTM 来源追踪系统** - 新增 `source` 字段记录流量来源
  - 推导逻辑：优先 `utm_source` 参数 → Referer 域名（`ref:domain`）→ `direct`
  - 数据库迁移：`click_logs.source`、`click_stats_hourly.source_counts`、`click_stats_daily.top_sources`
  - 异步处理：URL 解析、域名提取不阻塞 307 响应
- **IPC 配置段** - 新增 `[ipc]` 配置支持
  - `enabled`：启用/禁用 IPC 服务器
  - `socket_path`：自定义 socket 路径
  - `timeout`/`reload_timeout`/`bulk_timeout`：各类操作超时配置
  - `max_message_size`：最大消息大小（默认 64KB）
- **CLI 全局参数** - `--socket` 参数覆盖配置文件中的 socket 路径

### Changed
- **CLI 命令重构** - `generate-config` 改为 `config generate`，统一子命令风格
  - 新增 `--force` 参数强制覆盖现有配置文件

### Improved
- **重定向性能优化** - UTM 解析、GeoIP 查询全部移到后台 spawn，不阻塞 307 响应
- **存储空间优化** - 删除 `click_logs.user_agent` 列，只保留 hash 引用

### Refactored
- **TUI 状态管理** - 引入 `FormState` 结构体和 `EditingField` 枚举，取代零散的输入状态字段
  - 表单验证逻辑统一到 `validation.rs`
  - UI 组件（`add_link.rs`、`edit_link.rs` 等）大幅简化
- **Analytics 模块** - Rollup 汇总逻辑新增 `source_counts` 处理

### Dependencies
- 升级 `anyhow` 1.0.100 → 1.0.101
- 新增 `urlencoding` 2.1.3（URL 参数解码）

### Docs
- **文档结构重组** - 配置、部署、API 文档拆分为细粒度文件
  - 配置指南：`startup.md`、`runtime.md`、`security.md`、`examples.md`
  - 部署指南：Docker/Proxy/Systemd 各拆分为快速入门和运维文档
  - API 文档：Admin/Health 各拆分为端点详解
- 新增事件系统文档（`events/README.md`）

### Breaking Changes
- **CLI 命令变更** - `generate-config` → `config generate`
- **数据库 Schema 变更** - 新增 `source` 相关字段，删除 `click_logs.user_agent` 列
- **API 响应变化** - `/admin/v1/stats` 的 `referrers` 现在返回 `source`（utm_source / ref:{domain} / direct）

## [v0.5.0-alpha.6] - 2026-02-06

### 🎉 Release Highlights

v0.5.0-alpha.6 是一次可观测性增强版本，主要亮点：

- **Prometheus 指标系统** - 全新 18 项指标，涵盖 HTTP、数据库、缓存、重定向、认证等维度
- **Docker 多变体构建** - 标准版和 metrics 版分离，按需选择
- **事件系统文档重构** - 为插件化架构做准备，引入 trait-based Event 系统设计

### Added
- **Prometheus 指标导出** - 新增 `/health/metrics` 端点，导出 Prometheus 文本格式指标
  - HTTP：请求延迟直方图、请求计数、活跃连接数
  - 数据库：查询延迟、查询计数
  - 缓存：操作延迟、条目数、命中/未命中计数
  - 重定向：按状态码（307/404/500）统计
  - 点击：缓冲区大小、刷盘计数（按触发方式）
  - 认证：鉴权失败计数
  - Bloom Filter：假阳性计数
  - 系统：运行时间、进程内存、CPU 时间、构建信息
- **TimingMiddleware** - 自动记录所有 HTTP 请求的延迟和计数，使用 Drop Guard 确保活跃连接数准确
- **系统指标收集器** - 使用 sysinfo 库每 15 秒更新进程内存和 CPU 时间
- **指标辅助宏** - `observe_duration!`、`inc_counter!`、`set_gauge!` 等宏减少样板代码
- **Docker metrics 变体** - 新增 `latest-metrics`、`stable-metrics`、`edge-metrics` 等镜像标签

### Improved
- **API 响应格式统一** - 所有端点响应统一为 `{code, message, data}` 格式
- **健康检查增强** - 新增缓存状态信息和 uptime 指标
- **ObjectCache trait** - 新增 `entry_count()` 方法用于指标收集

### Refactored
- **指标记录优化** - 移除不必要的字符串克隆，端点分类返回 `&'static str` 避免堆分配
- **事件系统设计** - 文档重构为 trait-based Event 系统，支持事件取消和优先级机制

### Dependencies
- 添加 `prometheus` 0.14（可选，metrics feature）
- 添加 `sysinfo` 0.38（可选，metrics feature）
- 升级 `ureq` 3.1.4 → 3.2.0
- 升级 `time` 0.3.45 → 0.3.47
- 升级 `darling` 0.20.11 → 0.23.0

### CI/CD
- **Docker 多变体构建** - 构建矩阵支持 default 和 metrics 两个变体
- **镜像标签策略** - 新增 edge/stable 标签，metrics 版添加 `-metrics` 后缀
- **VitePress 文档** - 新增标签触发构建

### Docs
- 新增 `/health/metrics` 端点文档
- 新增 Docker metrics 版部署说明
- 重构事件系统文档，引入插件化架构设计
- 更新 API 响应格式说明

### Breaking Changes
- **最低 Rust 版本** - 要求 Rust 1.88+
- **API 响应格式** - 从 `{code, data}` 变更为 `{code, message, data}`

## [v0.5.0-alpha.5] - 2026-02-05

### 🎉 Release Highlights

v0.5.0-alpha.5 是一次代码质量和安全性提升版本，主要亮点：

- **请求 ID 中间件** - 每个请求分配唯一 UUID，注入日志 span 和响应头，便于追踪调试
- **缓存健康检查** - 健康检查端点新增缓存状态信息（类型、Bloom filter、Negative cache 状态）
- **批量操作安全增强** - 新增 5000 条批量大小限制和 10MB 导入文件限制
- **关机超时机制** - 优雅关闭增加 30 秒总超时保护，防止关闭卡死

### Added
- **Request ID 中间件** - 为每个请求生成 UUID v4，注入 tracing span 和 `X-Request-ID` 响应头
- **缓存健康检查** - `/health` 端点现在返回缓存状态（类型、Bloom filter、Negative cache）
- **Refresh Token 限流** - 每 10 秒 1 次请求，突发最多 10 次，防止滥用
- **批量操作大小限制** - 批量创建/更新/删除最多 5000 条
- **导入文件大小限制** - CSV 导入最大 10MB
- **新增错误码** - `BatchSizeTooLarge`、`FileTooLarge`、`InvalidDateFormat`、`LinkInvalidCode`、`LinkReservedCode`

### Improved
- **关机流程** - 增加超时机制（30 秒总超时，单任务 10 秒），超时强制退出防止卡死
- **日期参数验证** - `created_after`/`created_before` 参数无效时返回明确错误信息
- **JWT Service 缓存** - 使用 `OnceLock` 缓存实例，避免每次请求重复创建
- **登录日志** - 登录成功/失败日志包含客户端 IP
- **Bloom filter 初始容量** - 改为 100（启动时 reconfigure 会用实际数量替换），减少初始内存占用

### Security
- **Health Token 常量时间比较** - 使用 `subtle::ConstantTimeEq` 防止时序攻击
- **Admin Token 生成增强** - 默认使用加密安全的 `OsRng` 生成 32 字符令牌

### Refactored
- **移除 AdminService 包装层** - 直接使用函数，减少间接调用
- **IP 提取逻辑统一** - 登录限流和其他地方共用 `utils/ip.rs` 中的函数
- **批量操作重构** - 使用 `LinkService` 统一业务逻辑层
- **小时汇总写入器** - 新增 `HourlyRollupWriter` 统一 click_sink 和 rollup 的汇总逻辑
- **配置管理代码** - 使用宏简化 `get_runtime_config_or_return!()` 重复模式
- **前端服务** - 统一 `serve_index_html()` 逻辑，消除代码重复

## [v0.5.0-alpha.4] - 2026-02-05

### 🎉 Release Highlights

v0.5.0-alpha.4 是一次 Analytics 系统的重大升级，主要亮点：

- **点击数据汇总系统** - 多级汇总架构（小时/天/全局），大数据量查询性能提升 10-100 倍
- **UserAgent 去重存储** - xxHash64 去重 + woothee 解析，存储减少 30-70%
- **设备分析 API** - 新增浏览器、操作系统、设备类型统计端点
- **自动数据清理** - 可配置的保留策略，防止存储无限增长

### Added
- **点击数据汇总系统** - 实时更新的多级汇总架构
  - `click_stats_hourly`：每个短链接每小时的点击数、来源分布、国家分布
  - `click_stats_daily`：每个短链接每天的点击数、唯一来源数、唯一国家数
  - `click_stats_global_hourly`：全局每小时总点击数、活跃链接数
  - 后台任务自动将小时汇总滚动到天汇总
- **UserAgent 去重存储系统** - 新增 `user_agents` 表和 `UserAgentStore` 服务
  - 使用 xxHash64 生成 16 字符 hex hash 作为唯一标识
  - 使用 woothee 解析浏览器、操作系统、设备类型
  - DashMap/DashSet 高性能并发缓存
  - 后台任务每 30 秒批量写入新 UA
  - 支持历史数据自动迁移和字段回填
- **设备分析 API** - 两个新端点
  - `GET /admin/v1/analytics/devices` - 全局设备分析
  - `GET /admin/v1/links/{code}/analytics/devices` - 单链接设备分析
  - 返回浏览器、操作系统、设备类型分布和 Bot 占比
- **自动数据清理任务** - 可配置的保留策略
  - 原始点击日志：默认 30 天
  - 小时汇总：默认 7 天
  - 天汇总：默认 365 天
  - 后台任务每 4 小时运行，分批删除避免长事务
- **新增配置项**
  - `analytics.enable_auto_rollup` - 启用自动汇总和清理（默认 true）
  - `analytics.hourly_retention_days` - 小时汇总保留天数（默认 7）
  - `analytics.daily_retention_days` - 天汇总保留天数（默认 365）

### Improved
- **Analytics V2 查询方法** - 从汇总表读取的高性能查询
  - `get_trends_v2()`、`get_link_trends_v2()` 等方法提升 10-100 倍性能
  - 游标分页导出替代 OFFSET，大数据量导出性能显著提升
- **Click Sink 增强** - 每次 flush 时自动更新汇总表
- **查询表达式统一** - 排序和分组使用统一的表达式克隆，避免字符串表达式

### Refactored
- **启动流程重构** - 新增 UserAgentStore 初始化、历史数据迁移、UA 后台任务、数据清理任务
- **日志消息语言统一** - 从中文统一为英文

### Dependencies
- 添加 `xxhash-rust` 用于 UA hash 生成
- 添加 `woothee` 用于 UA 解析

### Docs
- 新增设备分析 API 文档
- 更新配置文档，添加汇总和清理相关配置项说明
- 更新 `analytics.log_retention_days` 说明（自动清理已实装）

## [v0.5.0-alpha.3] - 2026-02-04

### 🎉 Release Highlights

v0.5.0-alpha.3 是一次依赖优化版本，主要亮点：

- **HTTP 客户端替换** - 将 reqwest 替换为轻量级的 ureq，显著减少依赖体积
- **GeoIP 服务改进** - 改进日志信息，增强错误处理
- **Docker 标签策略优化** - alpha/beta/rc 版本不再打 latest 标签

### Changed
- **HTTP 客户端迁移** - 将 GeoIP 外部 API 调用从 reqwest 迁移到 ureq
  - ureq 是同步轻量级 HTTP 客户端，通过 `spawn_blocking` 异步执行
  - 移除 hyper、quinn (QUIC)、tower 等重量级依赖，减小编译体积
- **Docker 镜像标签策略** - 优化预发布版本的标签行为
  - alpha/beta/rc 版本仅打版本号标签（如 `v0.5.0-alpha.3`）
  - 正式版本打 `latest`、`stable` 和主版本号标签（如 `v1`）

### Improved
- **GeoIP 服务日志** - 错误日志中显示完整 URL，便于调试
- **GeoIP API 响应处理** - 正确处理 ip-api.com 返回的 `status: fail` 状态
- **文档版本显示** - VitePress 导航栏动态显示当前版本号

### Fixed
- **GeoIP API 默认 URL** - 增加 `status` 字段请求参数，确保能检测 API 失败状态

### Dependencies
- 替换 `reqwest` (0.13) 为 `ureq` (3.1.4)
- 移除 hyper、hyper-util、quinn、tower、tower-http 等间接依赖

### Tests
- 新增 4 个 GeoIP 外部 API 测试（标记 `#[ignore]` 因依赖外部网络）

## [v0.5.0-alpha.2] - 2026-02-04

### 🎉 Release Highlights

v0.5.0-alpha.2 是一次功能增强版本，主要亮点：

- **详细点击分析** - 新增 click_logs 表记录完整点击信息，支持来源、User-Agent、IP 和地理位置
- **Analytics API** - 6 个分析端点，支持趋势图、热门链接、来源统计、地理分布
- **GeoIP 服务** - 支持 MaxMind 本地数据库和外部 API 两种查询方式
- **配置系统增强** - 支持 `SL__` 前缀环境变量覆盖启动配置

### Added
- **click_logs 数据表** - 存储详细点击日志（short_code、clicked_at、referrer、user_agent、ip_address、country、city）
  - 添加 short_code、clicked_at 和复合索引优化查询性能
- **Analytics API 模块** - 新增 `/admin/v1/analytics` 端点组
  - `GET /analytics/trends` - 点击趋势（支持按小时/天/周/月分组）
  - `GET /analytics/top` - 热门链接排行
  - `GET /analytics/referrers` - 来源统计
  - `GET /analytics/geo` - 地理位置分布
  - `GET /analytics/export` - 流式导出 CSV 报告（支持大文件）
  - `GET /links/{code}/analytics` - 单链接详细统计
- **GeoIP 服务** - 支持两种地理位置查询方式
  - MaxMind GeoLite2 数据库（本地查询，高性能）
  - 外部 API 回退（带 LRU 缓存，10000 条，TTL 15 分钟）
- **环境变量配置覆盖** - 使用 `SL__` 前缀和 `__` 分隔符覆盖启动配置
  - 例如：`SL__SERVER__PORT=8080` 覆盖 `[server] port`
- **新增配置项**
  - `analytics.enable_detailed_logging` - 启用详细日志记录（默认 false）
  - `analytics.log_retention_days` - 日志保留天数（默认 30）
  - `analytics.enable_ip_logging` - 记录 IP 地址（默认 true）
  - `analytics.enable_geo_lookup` - 启用地理位置查询（默认 false）
- **IP 工具模块** - 新增 `src/utils/ip.rs`，提供私有 IP 检测、CIDR 匹配等功能

### Changed
- **配置加载方式** - 从 `dotenv` 迁移到 `config` + `dotenvy`，支持更灵活的配置优先级
- **重定向服务增强** - 支持记录详细点击信息，跳过私有/本地 IP 的 GeoIP 查询

### Improved
- **CSV 导出流式化** - 分析报告导出改为流式响应，支持大文件，避免内存溢出
- **分析服务层抽象** - 提取 AnalyticsService 统一业务逻辑，支持多接口复用
- **CSV 导出安全性** - 使用 csv crate 防止 CSV 注入攻击

### Refactored
- **IP 处理逻辑统一** - 将 `is_private_or_local`、`is_trusted_proxy`、`ip_in_cidr` 提取到独立模块
- **客户端 IP 提取** - 使用统一的 `extract_client_ip` 函数替代内联实现

### Dependencies
- 添加 `maxminddb` (0.27) 用于 MaxMind GeoLite2 数据库解析
- 添加 `reqwest` (0.13) 用于外部 GeoIP API 调用
- 添加 `config` (0.15) 用于配置文件加载
- 替换 `dotenv` 为 `dotenvy` (0.15)
- 升级 `criterion` 0.5 → 0.8
- 升级 `nix` 0.30 → 0.31
- 升级 `ts-rs` 11.1 → 12.0
- 升级 `socket2`、`windows-sys` 等依赖

### Docs
- 完善 Analytics API 文档，补充查询参数说明和 GeoIP 配置要求
- 更新配置文档，说明环境变量覆盖和配置优先级
- 更新 Docker、systemd 部署文档

## [v0.5.0-alpha.1] - 2026-02-02

### 🎉 Release Highlights

v0.5.0-alpha.1 是一次架构级别的重大重构版本，主要亮点：

- **配置系统重构** - 移除 config.toml 和环境变量支持，所有运行时配置现在存储在数据库中
- **错误码系统重构** - 统一 API 错误响应格式，提供更精确的错误类型
- **新增配置类型** - 支持 StringArray 和 EnumArray 配置类型

### Added
- **StringArray 和 EnumArray 配置类型** - 新增两种配置类型用于数组值
  - `StringArray`：用于字符串数组（如 `api.trusted_proxies`、`cors.allowed_origins`）
  - `EnumArray`：用于枚举数组（如 `cors.allowed_methods`），验证数组元素是否在允许的选项中
- **JSON 配置解析辅助方法** - `RuntimeConfig::get_json_or()` 方法，安全解析 JSON 配置并提供默认值
- **Admin API 错误码模块** - 新增 `error_code.rs`，集中定义 API 错误码

### Changed
- **配置系统架构重构** - 移除 config.toml 和环境变量支持
  - 所有运行时配置现在存储在数据库中
  - 将 `AppConfig` 拆分为 `StaticConfig`（静态基础设施配置）和 `RuntimeConfig`（运行时配置）
  - 启动时通过 `ConfigStore::ensure_defaults()` 初始化默认值
  - 移除配置迁移模块，简化启动流程
- **错误码系统重构** - 统一 API 错误响应格式
  - 使用 `thiserror` 派生宏重构错误类型
  - 每个错误类型关联唯一的错误码（如 `link_not_found`、`validation_error`）
  - 中间件和服务层统一使用新的错误类型

### Improved
- **配置 Schema 静态缓存** - 避免重复计算，提升性能
- **枚举选项生成逻辑** - 基于 `RustType` 自动推断，增加编译期安全检查
- **CORS 配置动态加载** - 从 RuntimeConfig 动态读取，支持热更新

### Fixed
- **TUI 过期时间解析错误类型** - 从 `validation` 更改为 `link_invalid_expire_time`，提高错误信息准确性

### Refactored
- **LinkService 简化** - 移除冗余的错误处理代码，使用新的错误类型
- **中间件错误处理** - 统一使用 `ShortlinkerError` 类型
- **配置更新逻辑** - 移除 AppConfig 同步，仅更新数据库和内存缓存

### Dependencies
- 添加 `thiserror` 依赖用于错误派生

### Migration Notes

**⚠️ 从 v0.4.x 升级注意事项：**

1. **配置系统变更** - `config.toml` 中的运行时配置项不再生效
   - 所有运行时配置现在存储在数据库中
   - 首次启动时会自动初始化默认值
   - 后续通过管理面板或 API 修改配置
2. **API 错误响应格式变更** - 错误响应现在包含更精确的 `code` 字段
   - 如 `link_not_found`、`validation_error`、`unauthorized` 等
   - 前端需要根据新的错误码处理错误

## [v0.4.3] - 2026-02-01

### 🎉 Release Highlights

v0.4.3 是一次紧急修复版本：

- **CSRF Cookie 路径修复** - 修复前端无法读取 CSRF cookie 导致所有变更操作（创建、修改、删除）返回 403 的问题

### Fixed
- **CSRF Cookie 路径问题** - 将 CSRF cookie 的 path 从 `admin_prefix` 改为 `/`
  - 修复前端页面路径与 cookie path 不匹配时，所有 POST/PUT/DELETE 请求返回 403 Forbidden 的问题
  - 确保任意 `admin_prefix` 配置下前端都能正常工作
  - 安全性不受影响（`SameSite=Lax` 仍然生效）

## [v0.4.2] - 2026-02-01

### 🎉 Release Highlights

v0.4.2 是一次针对性的安全与易用性改进版本，主要亮点：

- **智能代理检测** - 默认自动信任来自私有 IP（RFC1918）或 localhost 的连接，简化 Docker/nginx 反向代理部署
- **增强 Unix Socket 支持** - 强制要求 X-Forwarded-For 头部，防止登录限流失效
- **完善 IPv6 支持** - 扩展 IPv6 私有地址检测范围（ULA + 链路本地地址）

### Added
- **智能代理检测模式** - 未配置 `api.trusted_proxies` 时，自动信任来自私有 IP 或 localhost 的连接
  - 支持 RFC1918 私有地址段：`10.0.0.0/8`、`172.16.0.0/12`、`192.168.0.0/16`
  - 适合常见的 Docker、nginx、Caddy 反向代理场景，无需手动配置
  - 公网 IP 默认不信任 X-Forwarded-For，防止伪造攻击
- **启动时代理检测模式日志** - 显示当前使用的代理检测策略（Unix Socket / 显式配置 / 智能检测 / 直连），便于部署调试

### Improved
- **Unix Socket 模式增强** - 强制要求 X-Forwarded-For 头部，缺失时返回明确的错误提示
  - 防止 Unix Socket 模式下登录限流失效（所有请求来自同一 peer_addr）
  - 错误提示包含 nginx 配置示例：`proxy_set_header X-Forwarded-For $remote_addr;`
- **限流 key 提取器逻辑优化** - 按优先级处理（Unix Socket > 显式配置 > 智能检测 > 连接 IP）
  - 使用 `SocketAddr` 解析替代 `IpAddr`，支持带端口的 IP 地址
  - 优化 IP 地址解析流程：先尝试 `SocketAddr::parse()`，失败时回退到 `IpAddr::parse()`
- **IPv6 私有地址检测** - 扩展 IPv6 私有地址范围
  - 新增 `fc00::/7` (ULA, RFC 4193)：`fc00::/8` + `fd00::/8`
  - 新增 `fe80::/10` (链路本地地址)
  - 改进代码注释，明确各地址段定义及对应 RFC 标准

### Fixed
- **Unix Socket 模式警告重复** - 修复启动时代理检测警告逻辑，避免重复日志输出

### Docs
- 更新配置文档 `api.trusted_proxies` 说明
  - 详细说明智能检测和显式配置的使用场景
  - 添加安全提示：公网 IP 默认不信任 X-Forwarded-For
  - 添加 Docker/nginx 部署示例

### Migration Notes

**⚠️ 从 v0.4.1 升级注意事项：**

1. **默认行为变更** - 未配置 `api.trusted_proxies` 时，现在会自动信任来自私有 IP 的连接
   - 大部分反向代理场景（Docker、nginx）可直接使用，无需配置
   - 如需禁用智能检测，请显式配置 `api.trusted_proxies = []`（空数组）
2. **Unix Socket 模式更严格** - 现在强制要求 X-Forwarded-For 头部，请检查代理配置

## [v0.4.1] - 2026-02-01

### 🎉 Release Highlights

v0.4.1 是一次重要的安全与性能优化版本，主要亮点：

- **CSRF 防护** - 双令牌模式的 CSRF 中间件，防止跨站请求伪造攻击
- **流式 CSV 导出** - 支持大规模数据导出，内存占用从 O(N) 降至 O(batch_size)
- **登录限流安全增强** - 可信代理配置，防止 IP 伪造绕过限流
- **CSV 导入性能优化** - Bloom Filter 预筛选 + 批量查询，显著提升冲突检测性能
- **全面的测试覆盖** - 新增 8 个基准测试和覆盖 22 个模块的单元测试

### Added
- **CSRF 防护中间件** - 双令牌模式，验证 `X-CSRF-Token` header 与 Cookie 匹配
  - 使用 `subtle::ConstantTimeEq` 进行常量时间比较，防止时序攻击
  - 安全方法（GET/HEAD/OPTIONS）和 Bearer Token 认证自动跳过
  - 认证端点（login/refresh/logout）自动跳过 CSRF 验证
- **流式 CSV 导出** - 分批次流式导出（每批 10,000 条），支持 `Transfer-Encoding: chunked`
  - 使用 `spawn_blocking` 将 CSV 序列化移到独立线程池，避免阻塞 worker 线程
  - 内存占用优化：导出 100 万条链接内存占用从 O(N) 降至 O(1000)
- **可信代理配置** - 新增 `api.trusted_proxies` 配置项（支持 IP 和 CIDR 格式）
  - 登录限流 key 优先使用连接 IP（不可伪造），仅可信代理时使用 `X-Forwarded-For`
  - 支持 IPv4/IPv6 CIDR 匹配（如 `192.168.1.0/24`）
- **批量短码存在性检查 API** - `batch_check_codes_exist()` 方法，支持分批查询（每批 500 个）
- **API 常量模块** - 硬编码 Cookie 名称（`shortlinker_access`, `shortlinker_refresh`, `csrf_token`）
- **全面的基准测试套件** - 新增 8 个基准测试，覆盖缓存、IPC、密码哈希等关键路径
  - `cache_layer`: CompositeCache 各操作性能
  - `import_conflict`: CSV 导入冲突检测策略对比
  - `ipc_protocol`: IPC 协议序列化/反序列化性能
  - `password`: Argon2 密码哈希性能
  - `utils`: 短码生成、URL 验证等工具函数性能
- **单元测试覆盖** - 新增测试覆盖 22 个模块（JWT、缓存层、存储层、服务层、错误处理等）
- **集成测试套件** - 新增 `link_service_tests.rs`（865 行）和 `storage_tests.rs`（695 行）

### Changed
- **Cookie 安全标志默认开启** - `cookie_secure` 默认改为 `true`，强制 HTTPS 传输 Cookie
  - `cookie_secure=false` 时启动输出警告日志
- **移除可配置 Cookie 名称** - 移除 `access_cookie_name` 和 `refresh_cookie_name` 配置项
  - 改为硬编码常量，减少攻击面
- **升级依赖** - Rust 1.93-slim，移除 OpenSSL 依赖（使用 rustls）

### Improved
- **CSV 导入冲突检测性能优化** - 全量加载 -> Bloom Filter 预筛选 -> 批量查询
  - 预扫描 CSV 提取所有 codes，使用 Bloom Filter 快速排除肯定不存在的 codes
  - 仅对可能存在的 codes 执行批量数据库查询
- **CSV 导出流式化** - 使用 `stream::unfold` 实现分页流式查询（每批 1000 条）
- **Dockerfile 构建优化** - 减少镜像层大小，静态链接编译，移除未使用的依赖
- **查询条件构建** - 使用 SeaORM `contains()` 方法替代手动字符串拼接，防止通配符注入
- **基准测试性能** - 手动创建运行时，避免每次迭代创建/销毁

### Fixed
- **健康检查未授权响应** - 统一业务码为 1（之前为 401），保持 API 响应格式一致性
- **Symlink 攻击防护** - 使用 `create_new()` 原子创建 `admin_token.txt`，防止 TOCTOU 竞态条件
- **SeaORM contains() 误用** - 移除手动添加 `%` 通配符，`contains()` 方法已自动处理

### Security
- **IPC 权限控制** - Unix socket 文件创建后设置权限为 `0600`（仅属主可读写）
- **Admin Token 文件安全** - 原子创建文件 + Unix 权限 0600，防止 symlink 攻击
- **登录限流增强** - 默认使用连接 IP（不可伪造），防止客户端伪造 IP 绕过限流
- **CSRF Cookie SameSite 设置** - CSRF Cookie 使用 `Lax` 模式，防止跨站 POST 请求
- **认证方式显式标记** - 在 `req.extensions()` 中插入 `AuthMethod`，CSRF 中间件根据标记判断是否跳过验证

### Dependencies
- 升级 Rust 至 1.93-slim
- 移除 OpenSSL 依赖（使用 rustls）

### Docs
- 更新 Admin API 鉴权文档，新增 CSRF 防护说明
- 更新配置文档，新增 `api.trusted_proxies` 说明

### Migration Notes

**⚠️ 从 v0.4.0 升级注意事项：**

1. **CSRF 防护默认启用** - Web 管理面板的所有变更操作需携带 `X-CSRF-Token` header
2. **Cookie 名称硬编码** - `access_cookie_name` 和 `refresh_cookie_name` 配置项已移除，现在固定为 `shortlinker_access` 和 `shortlinker_refresh`
3. **Cookie Secure 默认开启** - 如需在非 HTTPS 环境使用，需显式设置 `cookie_secure=false`（启动时会有警告）
4. **可信代理配置** - 如果在代理/负载均衡器后部署，建议配置 `api.trusted_proxies` 列表以正确识别客户端 IP

## [v0.4.0] - 2026-01-23

### 🎉 Release Highlights

v0.4.0 是一次架构级别的重大更新，主要亮点：

- **IPC 跨进程通信系统** - 全新的 Unix 域套接字 / Windows 命名管道通信机制，CLI 可直接与运行中的服务器交互
- **TUI 全面重构** - 组件化架构、模糊搜索、批量操作、详情面板、排序功能
- **CSV 导入导出** - 统一的 CSV 格式，替代原有 JSON 格式（JSON 将在 v0.5.0 移除）
- **服务层抽象** - 新增 LinkService 统一业务逻辑，消除代码重复

### Added
- **IPC 跨进程通信系统** - 替代原有的 Unix 信号和 Windows 文件轮询机制
  - Unix 域套接字 (`/tmp/shortlinker.sock`) 和 Windows 命名管道 (`\\.\pipe\shortlinker`)
  - 长度前缀 JSON 协议，支持链接 CRUD、导入导出、状态查询等命令
  - CLI 命令优先通过 IPC 与服务器通信，确保缓存同步
- **CLI `status` 命令** - 通过 IPC 查询服务器状态（版本、运行时间、链接数量等）
- **TUI 模糊搜索** - 使用 `nucleo-matcher` 实现智能匹配，按 `/` 进入搜索模式
- **TUI 批量操作** - 支持多选（Space 键）和批量删除
- **TUI 详情面板** - 右侧面板显示选中链接的完整信息，支持剪贴板复制
- **TUI 排序功能** - 按短代码、URL、点击量、状态排序
- **CSV 导入导出** - 统一 CLI、TUI、Web Admin 的导入导出格式
  - 自动检测文件格式（CSV/JSON）
  - JSON 格式已标记废弃
- **LinkService 服务层** - 统一的业务逻辑层，被 IPC 和 HTTP 处理器共享
- **重载协调器** - 新增 `ReloadCoordinator` 支持数据重载和配置热重载

### Changed
- **CLI IPC 优先架构** - 服务器运行时优先使用 IPC 通信，否则直接访问数据库
- **API 强类型响应** - 将动态 JSON 替换为强类型结构体，提高类型安全性
- **reset-password 命令改进** - 支持交互式密码输入（使用 `rpassword`）

### Improved
- **TUI 组件化架构** - 引入 Action 系统、Component trait、可复用 UI 组件
- **导入性能优化** - 使用批量查询替代循环单次查询
- **密码处理统一** - 新增 `password.rs` 工具模块，统一密码哈希处理逻辑

### Fixed
- **URL 验证增强** - 统一 CLI 和 TUI 的 URL 验证，阻止危险协议
- **导入密码处理** - 正确区分新密码和已哈希密码，阻止明文密码直接存储
- **Unix 守护进程启动** - 修复 IPC 无响应时错误清理 PID 文件的问题
- **导入结果统计** - 区分 `skipped` 和 `failed` 字段

### Refactored
- **TUI 状态管理** - 拆分 `state.rs` 为 `form_state.rs` 和模块化状态管理
- **错误定义宏** - 使用宏重构 `ShortlinkerError` 枚举
- **平台层简化** - 移除旧的 `reload.rs`，功能迁移到 `reload/` 模块

### Dependencies
- 添加 `bytes` (1.11) 用于 IPC 协议编解码
- 添加 `arboard` (3.6) 用于 TUI 剪贴板支持
- 添加 `nucleo-matcher` (0.3) 用于 TUI 模糊搜索
- 添加 `rpassword` (7) 用于 CLI 密码输入

### Docs
- 更新 CLI 命令文档以反映 IPC 优先行为
- 更新 TUI 帮助文档以反映新增功能快捷键
- 新增 `TUI_REFACTOR_REPORT.md` 记录架构改进

### Migration Notes

**⚠️ 从 v0.3.x 升级注意事项：**

1. CLI 命令现在优先通过 IPC 与运行中的服务器通信，确保缓存同步
2. 导入导出默认使用 CSV 格式，JSON 格式仍可读取但已标记废弃
3. IPC 套接字路径：Unix `/tmp/shortlinker.sock`，Windows `\\.\pipe\shortlinker`

## [v0.3.0] - 2026-01-19

### 🎉 Release Highlights

v0.3.0 是一个重大版本更新，包含大量安全增强、性能优化和新功能。主要亮点：

- **动态配置系统** - 支持运行时热更新配置，无需重启服务
- **JWT 认证** - 完整的访问令牌/刷新令牌认证体系
- **负向缓存** - 优化缓存架构，减少无效查询
- **管理员登录限流** - 防止暴力破解攻击
- **敏感信息保护** - 全面加强日志和数据库中的敏感信息处理

### Added
- **动态配置系统** - 基于数据库的运行时配置管理，支持配置热重载
  - 新增 `system_config` 和 `config_history` 数据表
  - 配置管理 API 端点（`GET/PUT /admin/v1/config`）
  - 首次启动自动从 `config.toml` 迁移配置到数据库
- **配置管理 CLI 命令** - `config list/get/set/reset/export/import`
- **配置 Schema 系统** - 支持前端动态渲染配置表单，按类别分组展示
- **JWT 认证系统** - 替换 Bearer Token，支持 Access/Refresh Token
- **健康检查 Bearer Token 认证** - 支持 k8s 等监控工具通过 `HEALTH_TOKEN` 访问
- **负向缓存（Negative Cache）** - 缓存不存在的键，减少数据库压力
- **数据库连接重试机制** - 指数退避重试策略，增强连接稳定性
- **链接导出导入功能** - CSV 格式批量导出导入，支持冲突处理模式
- **管理员登录限流** - 基于 IP 的速率限制（1 req/s，burst 5），防止暴力破解
- **短码格式验证** - 长度≤128，字符集 `[a-zA-Z0-9_.-/]`
- **自定义前端支持** - `./frontend-panel` 目录替换内置管理面板
- **ClickManager 基准测试** - criterion 性能测试套件
- **cargo-binstall / Homebrew / macOS x86_64 构建支持**

### Changed
- **CORS 默认禁用** - 提升默认安全性，需显式启用
- **缓存预热策略** - 从主动加载改为按需回填（cache-aside）
- **默认管理员令牌长度** - 从 8 位增加到 16 位
- **敏感配置掩码** - 统一使用 `[REDACTED]` 替代 `********`
- **Cookie 配置支持热更新** - `cookie_secure`、`cookie_same_site`、`cookie_domain`

### Improved
- **ClickManager 性能优化** - 添加原子标志消除任务风暴，优化 Arc 分配（单线程 -40%，并发 +40%~+62%）
- **批量接口性能** - N+1 查询优化为 2 次 DB 往返
- **数据库错误处理统一** - 所有查询返回 `Result<Option<T>>`
- **健康检查性能** - 改用轻量级 `count()` 查询
- **启动流程优化** - 返回 Result 替代 panic，增加错误处理
- **日志初始化回退** - 文件日志失败时优雅回退到 stdout

### Fixed
- **敏感信息泄露修复**
  - 自动生成的 `admin_token` 不再打印到日志，改为写入 `admin_token.txt` 文件
  - 配置迁移时直接写入哈希值，避免明文先存入数据库
  - 配置历史表添加敏感 key 兜底列表，确保脱敏
- **SQL 注入防护** - `click_sink` 增加短码格式校验作为防御性检查
- **运行时配置加载** - 区分启动模式和热重载模式，启动时正确加载所有配置
- **ClickBuffer 数据竞争修复** - 通过先快照 key 再逐个删除避免竞态
- **刷盘失败恢复机制** - 失败时自动将数据写回缓冲区
- **缓存删除逻辑** - 使用负缓存标记已删除的键
- **CSV 导入重复短码** - 改用 HashMap 去重
- **CORS 配置** - 防止通配符源与凭据同时启用
- **Cookie 路径设置** - Access Cookie path 收窄到 `admin_prefix`

### Refactored
- **配置系统重构** - `definitions.rs` 作为配置项的唯一数据源
- **API 路由模块化** - admin、frontend、health、redirect 拆分为独立模块
- **CLI 使用 clap 重构** - 移除自定义参数解析器
- **点击缓冲区计数器** - 重命名为 `total_clicks`，跟踪总点击数
- **短码验证统一** - 提取到 `utils::is_valid_short_code()` 共用

### Dependencies
- 添加 `actix-governor`、`governor` 用于速率限制
- 添加 `clap` 用于命令行解析
- 添加 `strum` 用于 enum 派生
- 添加 `criterion` 用于基准测试
- SeaORM 升级至 2.0.0-rc.28
- actix-web 升级至 4.12

### Admin Panel
- **PWA 支持** - 可作为渐进式 Web 应用安装，支持自动更新检测
- **短代码路径格式** - 支持斜杠作为路径分隔符（如 `abc/def`）
- **短代码验证规则** - 允许点号，最大长度从 50 增加到 128
- **系统配置界面重构** - 新增分类展示和 UI 组件
- **i18n 改进** - 简化导入覆盖模式描述，改进 CORS 配置项翻译

### Migration Notes

**⚠️ 从 v0.2.x 升级注意事项：**

1. 首次启动时，系统会自动从 `config.toml` 迁移配置到数据库
2. 迁移完成后，数据库配置作为主配置源，`config.toml` 中的动态配置项不再生效
3. 自动生成的 `admin_token` 会写入 `admin_token.txt` 文件，请保存后删除
4. CORS 默认禁用，如需跨域访问请显式配置
5. 建议检查并更新 `admin_token` 为更强的密码

## [v0.3.0-beta.3] - 2026-01-18

### Added
- **ClickManager 性能基准测试** - 新增 criterion 基准测试套件
  - 包括单线程/多线程 increment、不同 key 场景和 drain 操作的性能测试
  - 添加并发 increment 和 increment+drain 场景的单元测试，验证数据一致性
- **cargo-binstall 支持** - 用户可通过 `cargo binstall shortlinker` 直接安装预编译二进制
- **Homebrew 发布支持** - 用户可通过 `brew install AptS-1547/tap/shortlinker` 安装
- **macOS x86_64 (Intel Mac) 构建** - Release 现已支持 Intel Mac 平台

### Fixed
- **ClickBuffer 数据竞争修复** - 重构 `drain` 方法，通过先快照 key 再逐个删除的方式避免数据竞争 (#40, #41)
- **刷盘失败恢复机制** - 新增 `restore` 方法，刷盘失败时自动将数据写回缓冲区，避免数据丢失
- **ClickBuffer 竞态条件修复** - 使用 `entry` API 重构 `increment` 方法，消除检查后插入（TOCTOU）的竞态条件
- **点击计数下溢防护** - 将 `fetch_sub` 替换为 `fetch_update`，确保总点击数减法操作不会下溢

### Docs
- 更新健康检查 API 鉴权逻辑文档
- 更新短链接路径格式约束说明
- 更新配置项说明：CORS 默认禁用、Cookie 配置热更新说明
- 更新存储配置文档：移除 `DATABASE_BACKEND` 说明，明确从 `DATABASE_URL` 自动推断
- 更新部署要求：Rust 版本提升至 1.85+ (Edition 2024)
- 同步更新英文文档

### Dependencies
- 添加 `criterion` 依赖用于基准测试
- 升级 `colored` 至 3.1.1

## [v0.3.0-beta.2] - 2026-01-16

### Added
- **负向缓存（Negative Cache）** - 缓存数据库中不存在的键，减少无效查询的数据库压力
  - 重构缓存查询流程：Bloom Filter → 负向缓存 → 对象缓存 → 数据库
  - 统一缓存结果枚举，将 `ExistsButNoValue` 重命名为 `Miss` 以更准确表达语义
- **数据库连接重试机制** - 新增指数退避重试策略，增强数据库连接稳定性
  - 配置项：重试次数、基础延迟、最大延迟
  - 为所有数据库操作（查询、插入、删除、点击计数刷新）添加重试逻辑
- **日志初始化回退机制** - 当文件日志初始化失败时，优雅地回退到 stdout 输出
- **健康检查独立 Token** - `health_token` 不再强制依赖 `admin_token`，支持独立配置
- **短码格式验证** - 重定向服务增加短码验证（长度≤128，字符集限制），TUI 同步更新规则

### Changed
- **CORS 默认禁用** - `default_cors_enabled` 的默认值从 `true` 改为 `false`，提升默认安全性
- **缓存预热策略调整** - 从主动加载改为按需回填（cache-aside），启动时仅加载短码到 Bloom Filter
- **默认管理员令牌长度** - 从 8 位增加到 16 位，增强安全性
- **时间格式显示优化** - 使用简洁的英文缩写（如 "2d 3h"）

### Improved
- **数据库错误处理统一** - 所有查询返回 `Result<Option<T>>`，防止静默失败
- **健康检查性能** - 改用轻量级 `count()` 查询替代 `load_all()`，避免内存压力
- **批量查询性能** - `batch_get_existing()` 使用 HashSet 提升查找性能
- **敏感信息保护** - 敏感配置更新时日志中隐藏明文值
- **Cookie 安全属性** - 清理时添加 secure、same-site 和 domain 属性
- **启动流程优化** - 增加错误处理和耗时日志，改进关机信号处理
- **SQLite 连接池** - 增加健康检查和连接超时设置
- **时间解析器** - 添加算术溢出检查，错误消息国际化为英文

### Fixed
- **缓存删除逻辑** - 修复 `CompositeCache::remove` 方法，删除对象缓存后使用负缓存标记键
- **CSV 导入** - 修复因重复短码导致的批量插入失败问题，改用 HashMap 去重
- **短链点击计数** - 修复创建时 `click_count` 字段初始化逻辑，使用传入的点击次数而非固定为 0
- **CORS 配置** - 添加安全验证，防止通配符源与凭据同时启用
- **配置更新函数** - `update_config_by_key` 遇到未知键时正确返回 `false`
- **Upsert 操作** - 包含 ClickCount 和 CreatedAt 字段，确保数据完整性

### Refactored
- **点击缓冲区计数器** - `ClickBuffer.counter` 重命名为 `total_clicks`，用于跟踪总点击数
- **前端静态资源加载** - 使用异步 IO 并统一加载逻辑

### Dependencies
- 更新 `aws-lc-rs`、`chrono`、`rust-embed`、`wasm-bindgen` 等依赖版本
- 升级 `rustc-demangle` 至 `0.1.27`，`unicode-truncate` 至 `2.0.1`
- 移除 `itertools 0.13.0`

### Admin Panel
- **PWA 更新检测** - 新增 `usePwaUpdate` hook，每小时自动检查 Service Worker 更新并显示通知提示
- **短代码路径格式** - 支持使用斜杠作为路径分隔符（如 `abc/def`）
- **短代码验证规则扩展** - 允许点号（`.`）作为有效字符，最大长度从 50 增加到 128
- **i18n** - 简化导入覆盖模式的描述文本，改进 CORS 配置项翻译说明

## [v0.3.0-beta.1] - 2026-01-15

### Added
- **配置管理 CLI 命令** - 新增完整的配置管理命令 (#37)
  - `config list`：列出所有配置项，支持按类别筛选和 JSON 输出
  - `config get`：获取指定配置项详细信息，包含类型、默认值等元数据
  - `config set`：设置配置值，支持验证和敏感信息掩码
  - `config reset`：重置配置到默认值
  - `config export`：导出所有配置到 JSON 文件
  - `config import`：从 JSON 文件导入配置，支持预览和强制覆盖
- **健康检查 Bearer Token 认证** - 支持 k8s 等监控工具通过 `HEALTH_TOKEN` 访问健康端点 (#38)
  - 认证流程：先尝试 Bearer token，再尝试 JWT Cookie
- **配置 Schema 系统** - 支持前端动态渲染配置表单
  - 为 Cookie SameSite 策略和 HTTP 方法添加类型安全的 enum 定义
  - 实现配置值验证器，确保 enum 配置值的合法性
  - 新增配置 Schema API 端点 (`/admin/v1/config/schema`)
- **配置分组功能** - 配置项按类别分组：认证、Cookie、功能开关、路由、CORS、点击追踪
- **自定义前端支持** - 可将自定义前端放入 `./frontend-panel` 目录替换内置管理面板 (#32)
  - 参数注入机制：自动替换 `%BASE_PATH%`、`%ADMIN_ROUTE_PREFIX%` 等占位符
- **自动生成初始管理员令牌** - 首次部署时自动生成 8 位随机令牌并在日志中提示保存 (#35)

### Changed
- **配置系统重构为单一数据源架构** - `definitions.rs` 作为配置项的唯一数据源
  - 集中定义所有配置的元信息（key、类型、默认值、分类等）
  - 配置迁移和 Schema 生成自动基于定义，减少重复代码
- **CLI 使用 clap 重构** - 移除自定义参数解析器，使用 clap derive 宏 (#33)
  - 删除 `src/config/args.rs` 和 `src/interfaces/cli/parser.rs`
  - 新增 `src/cli.rs` 定义 CLI 结构
- **Cookie 配置支持热更新** - `cookie_secure`、`cookie_same_site`、`cookie_domain` 的 `requires_restart` 改为 `false`
- **文档统一认证方式为 JWT Cookie** - 更新 Admin API 和 Health API 文档

### Improved
- **配置元数据同步机制** - 配置迁移时自动同步 `value_type`、`requires_restart`、`is_sensitive` 字段
- 新增 `strum` 依赖用于 enum 派生

### Docs
- 新增管理面板开发指南和故障排除文档
- 更新 CLI 文档，添加 `config` 命令说明
- 更新 README，添加自定义前端说明

### Dependencies
- 添加 `clap` (4.x) 用于命令行解析
- 添加 `strum` (0.27) 用于 enum 派生

## [v0.3.0-alpha.7] - 2026-01-14

### Added
- **链接导出导入功能** - 新增 CSV 格式的批量导出导入接口
  - 导出支持按时间、状态等条件筛选
  - 导入支持跳过 (skip)、覆盖 (overwrite)、报错 (error) 三种冲突处理模式
  - 新增 `/links/export` 和 `/links/import` API 端点
- **Admin Panel PWA 支持** - 管理面板可作为渐进式 Web 应用安装

### Improved
- **Redis 缓存批量失效** - 实现 `invalidate_all` 方法，通过 SCAN 命令批量删除指定前缀的缓存键
- **缓存容错性增强** - 反序列化失败时自动删除损坏的 Redis 缓存数据，防止持续读取错误
- **批量删除原子操作** - 批量删除短链接改为数据库事务原子操作，保证数据一致性

### Refactored
- **Redis 连接管理** - 使用 ConnectionManager 简化连接处理，移除手动连接池管理

### Dependencies
- 添加 `actix-multipart` 和 `csv` 依赖以支持文件上传和 CSV 处理
- 更新 `rust-embed` 和 `rust_decimal` 依赖版本

## [v0.3.0-alpha.6] - 2026-01-13

### Fixed (Admin Panel)
- **API 版本管理重构** - 版本号 (`/v1`) 改由前端自己管理，后端只注入路由前缀 (`/admin`)
- **Login 页面添加版本号** - 在登录页面底部显示版本信息，与 Sidebar 风格一致

## [v0.3.0-alpha.5] - 2026-01-13

### Added
- **搜索索引迁移** - 新增 `m020260112_000001_search_index` 迁移，为 `short_links` 表添加数据库特定的搜索索引
  - PostgreSQL：使用 pg_trgm 扩展和 GIN 索引支持高效的模糊搜索
  - MySQL：添加 FULLTEXT 索引支持全文搜索
  - 所有数据库：添加复合索引 `(expires_at, created_at DESC)` 优化未过期链接的排序查询
- **TypeScript 类型生成** - 集成 ts-rs 自动生成 TypeScript 类型定义

### Changed
- **Admin API 路径重构** - 统一添加 `/v1` 前缀
- **依赖升级**
  - SeaORM 升级至 2.0.0-rc.28
  - actix-web 升级至 4.12
  - ratatui 升级至 0.30.0
- 配置迁移模块使用强类型的 ValueType 枚举
- 健康检查服务响应结构优化，增强类型安全性
- 更新 admin-panel 子模块到最新提交

### Improved
- **性能优化**
  - 为 actix-web 启用 brotli/gzip 压缩中间件，减少网络传输
  - 使用 moka 缓存分页 COUNT 查询结果（TTL 30秒），提升分页性能
  - 重构统计查询为单次 DSL 聚合查询，减少数据库往返
  - 简化 upsert 实现，移除冗余的后备策略
  - 数据变更时自动清除 COUNT 缓存，保证一致性

### Fixed
- 修复 ratatui 0.30.0 的 API 变更：移除 Stylize trait 导入，更新 terminal.draw 错误处理
- 修复全局点击管理器重复初始化时的 panic，改为警告日志

### Refactored
- **CLI 模块拆分** - 将链接管理命令拆分为独立模块（add, list, update, remove, import_export, config_gen）
- **缓存错误处理重构** - Bloom 和 Null 过滤器构造函数返回 Result，移除 panic
- **统一缓存接口** - `ExistenceFilter::clear` 和 `CompositeCacheTrait::reconfigure` 返回 Result
- **SeaOrmStorage 重构** - 分离查询、变更和点击刷新逻辑到独立模块
- 配置项值类型使用 `Arc<String>` 避免重复克隆

### Docs
- 精简 README 内容，将 PNG 图片替换为 JPEG 格式以减小文件大小
- 更新 API 文档：添加搜索参数、统计接口、批量操作和认证端点
- 新增 TUI 模式、密码保护、批量操作和运行时配置 API 文档
- 更新健康检查 API 响应格式，统一为 code/data 结构
- 统一数据库配置：使用 DATABASE_URL 替代 STORAGE_BACKEND 和 DB_FILE_NAME

## [v0.3.0-alpha.4] - 2026-01-11

### Added
- **密码安全增强** - 使用 Argon2id 算法对管理员密码进行哈希处理
  - 支持从明文密码自动迁移到哈希密码
  - 新增 `reset-password` CLI 命令用于重置管理员密码
- **URL 安全验证** - 阻止危险协议（`javascript:`, `data:`, `file:` 等）的短链接创建
- **CORS 跨域支持** - 支持配置允许的源、方法和请求头
- **Docker Compose 配置** - 添加 `docker-compose.yml` 简化部署流程

### Changed
- **缓存 TTL 优化** - 基于链接过期时间动态调整缓存有效期，避免缓存过期链接
- **JWT 密钥管理** - 移除硬编码的 JWT_SECRET，改为自动生成安全随机密钥
- **配置迁移增强** - 支持增量迁移和密码自动升级

### Improved
- **性能优化**
  - 使用 `parking_lot` 替换 `tokio::sync::RwLock`，减少 Bloom Filter 锁开销
  - 使用 `Arc<str>` 替换 `String` 减少点击缓冲区中的字符串克隆开销
  - 优化点击计数批量更新，使用单条 SQL CASE 语句替代多次查询
- 更新配置迁移和运行时配置的日志信息，提升可读性

### Docs
- 更新配置文档，移除已废弃的配置项说明
- 移除 `redis` 缓存类型的过时注释（现已为有效配置项）

## [v0.3.0-alpha.3] - 2026-01-11

### Added
- **动态配置系统** - 实现基于数据库的运行时配置管理，支持配置热重载
  - 新增 `system_config` 和 `config_history` 数据表存储配置及变更历史
  - 添加配置管理 API 端点（`GET/PUT /api/admin/config`），支持通过 API 或管理面板动态修改配置
  - 支持配置热更新，无需重启服务即可应用大部分配置变更（JWT 密钥、API 令牌、路由前缀等）
  - 实现配置版本控制和变更历史追踪
  - 首次启动时自动从 `config.toml` 迁移配置到数据库
- 引入 `arc-swap` 依赖，实现配置的原子更新和线程安全访问

### Changed
- **配置架构重构** - 明确区分启动配置（需重启）和动态配置（可热更新）
  - 启动配置：数据库连接、服务器绑定地址、日志级别等
  - 动态配置：JWT 设置、API 令牌、CORS、路由前缀等功能配置
- 重构认证中间件和健康检查中间件，从运行时配置读取设置，支持动态更新
- 更新配置文档（`docs/config/index.md`），添加详细的配置说明和迁移指南
- 简化 `config.example.toml`，移除已迁移到数据库的动态配置项

### Improved
- 将错误测试从中文翻译为英文，提升代码国际化水平
- 修复配置文档中的代码示例，添加缺失的 `use` 语句

### Migration Notes
**⚠️ 重要：首次启动 v0.3.0-alpha.3 时，系统会自动从 `config.toml` 迁移配置到数据库**

1. 升级后首次启动时，系统会自动执行配置迁移
2. 迁移完成后，数据库配置将作为配置源，`config.toml` 中的动态配置项不再生效
3. 后续可通过管理面板或 API 修改配置，变更会实时生效且持久化到数据库
4. 如需重置为 `config.toml` 配置，请删除数据库后重新启动

查看完整配置文档：`docs/config/index.md`

## [v0.3.0-alpha.2] - 2026-01-07

### Changed
- chore(deps): bump preact

## [v0.3.0-alpha.1] - 2026-01-07

### Added
- JWT 认证支持

### Changed
- 升级 shortlinker 版本至 0.3.0-alpha.1
- 升级依赖并添加 JWT 认证支持
- 更新 admin-panel 子模块到新提交

### Refactored
- 统一 API 错误码格式，分离健康检查数据与响应结构
- 添加 JWT 认证系统替换 Bearer Token

## [v0.2.3-alpha.3] - 2025-10-24

### Changed
- Update Dockerfile

### Refactored
- 重构项目模块结构
- 重构 TUI 应用模块结构
- 重构 TUI 界面和构建配置

## [v0.2.3-alpha.2] - 2025-10-24

### Changed
- 更新版本号并完善 Docker 构建
- 更新依赖包并改进 TUI 功能

## [v0.2.3-alpha.1] - 2025-10-24

### Changed
- ORM 重构 (#25)
- chore(deps): bump vite (#24)

## [v0.2.2] - 2025-10-21

### Added
- 增强配置管理和日志系统

### Changed
- 优化构建流程并添加版本信息显示
- 清理已弃用代码并升级依赖
- chore(deps): bump vite (#23)

### Docs
- improve documentation accuracy and completeness

## [v0.2.1] - 2025-10-21

### Added
- 更新开发中功能状态，完成日志记录与轮转
- 增强日志系统和点击统计配置

### Changed
- 更新项目版本和依赖项
- 更新依赖并优化代码结构
- Update vitepress.yml

### Refactored
- 重构系统架构和平台抽象层
- 优化缓存接口参数和简化代码结构
- 优化点击管理器并发控制和性能监控
- 优化日志级别和代码结构

### Docs
- 更新文档以反映存储后端和 TUI 功能变化

## [v0.2.0] - 2025-09-01

### Added
- 添加完整的 TUI 终端用户界面
  - 支持链接的查看、添加、编辑、删除操作
  - 添加导入导出功能，支持 JSON 格式数据交换
  - 包含过期时间管理和密码保护等高级特性
  - 提供丰富的用户交互和状态反馈机制
- 添加 TUI 支持并改进错误处理
- 添加 TOML 配置文件支持
- 添加 Rust 开发相关的 VSCode 扩展

### Changed
- 更新依赖和构建配置
- 实现功能模块化构建配置
- 升级版本至 0.2.0-alpha.2 并优化代码

### Fixed
- 增加 flush 过程中的调试信息以改善日志可读性 (Fixed #14)

### Refactored
- 重构缓存系统命名和初始化逻辑
- 简化缓存配置机制
- 重构配置系统和移除废弃组件
- 重构点击统计后台任务启动方式 (Fixed #14)

## [v0.2.0-alpha.3] - 2025-08-30

### Added
- 添加完整的 TUI 终端用户界面
- 添加 TUI 支持并改进错误处理
- 添加详细的 TUI 使用文档

### Changed
- 实现功能模块化构建配置
- 升级版本至 0.2.0-alpha.2 并优化代码

### Refactored
- 重构缓存系统命名和初始化逻辑

## [v0.2.0-alpha.2] - 2025-08-27

### Added
- 添加 TOML 配置文件支持

### Fixed
- 增加 flush 过程中的调试信息以改善日志可读性 (Fixed #14)

### Refactored
- 简化缓存配置机制
- 重构配置系统和移除废弃组件

## [v0.2.0-alpha.1] - 2025-08-27

### Changed
- 更新依赖和构建配置

## [v0.1.7-alpha.7] - 2025-06-21

### Added
- 点击统计功能 (#17)

### Fixed
- 优化 Redis 客户端创建和缓存获取逻辑

### Reverted
- Revert "Feat click count (#15)" (#16)

## [v0.1.7-alpha.6] - 2025-06-21

### Added
- 添加 uuid 依赖并创建项目路线图文档

### Changed
- Refactor system and redis (#13)
- Update Dockerfile

### Fixed
- 更新依赖项版本并移除不再使用的服务器命令

## [v0.1.7-alpha.5] - 2025-06-19

### Added
- 添加对 MySQL 和 PostgreSQL 存储后端的支持
- 添加子模块支持并更新代码检出步骤
- 添加管理面板界面图片到文档

### Changed
- 更新 Dockerfile 和依赖项，提升构建效率和兼容性

### Fixed
- 移除 Dockerfile 中的 .git 目录并更新 admin-panel 子项目的提交引用
- 移除 admin-panel 中的 .git 目录并更新数据库文件名环境变量
- 更新管理面板子项目的提交引用
- 更新 bug 报告模板中的版本选项以反映最新版本
- 优化开发分支和预发布版本的判断逻辑
- 修改路由匹配规则以支持更灵活的链接代码格式

### Refactored
- migrate from rusqlite to sqlx for SQLite backend
- 删除前端文件夹

## [v0.1.7-alpha.4] - 2025-06-11

### Refactored
- 使用 rust-embed 重构前端静态文件服务

## [v0.1.7-alpha.3] - 2025-06-10

### Added
- 添加语言切换组件，支持日语和语言配置更新

### Changed
- enhance dark mode support in LoginView and update dependencies
- Enhance UI/UX for LinksView and LoginView with modern design elements

### Fixed
- 修复预启动处理时间输出，确保正确计算毫秒数

### Style
- 重命名函数和变量以提高可读性

## [v0.1.7-alpha.2] - 2025-06-08

### Added
- Add unit tests for Rust middleware and utils, plus Vue store and service (#9)
- Add docs for enabling admin panel (#8)

### Changed
- 更新 .gitignore，添加 .yarn/ 目录；删除 install-state.gz 文件

### Fixed
- 更新 OUT_DIR 环境变量错误提示；将 admin 面板构建命令从 npm 改为 yarn

### Refactored
- 优化启动流程，重构缓存和存储初始化逻辑；添加全局更新锁以防止并发更新

### Docs
- 更新基准测试版本号至 v0.1.7-alpha.1
- 更新管理面板和 Cloudflare Worker 文档，移除多余信息

## [v0.1.7-alpha.1] - 2025-06-08

### Added
- Add frontend build process and integrate with Rust backend
- 添加文件和 SQLite 存储插件，重构存储注册机制
- Implement layered caching with L1 and L2 cache plugins
- 添加 L1 和 L2 缓存插件的默认实现，重构缓存清理和加载逻辑
- Implement CORS support and admin authentication in the Actix web application
- update LoginView for improved UI and internationalization

### Changed
- 更新前端构建流程，使用 Node.js 20，改为 npm，增加版本号支持
- Update README.md

### Fixed
- 使用 vec 保证 get_all_links 的排列顺序

### Refactored
- 重构缓存系统架构并引入点击追踪
- 重构缓存加载逻辑，合并 L1 和 L2 缓存加载方法
- 重构存储插件注册机制，使用宏简化插件注册过程
- Middleware 重构

### Docs
- 更新 README.md 中的 CPU 信息和基准测试结果
- 更新 README.md 格式，优化环境和配置变量部分的可读性

## [v0.1.6] - 2025-06-06

### Added
- 添加布隆过滤器支持，优化链接重定向性能
- 在 CLI 模式下跳过布隆过滤器检查
- 在 SQLite 存储中添加布隆过滤器初始化逻辑 (fix #5)

### Changed
- 将存储工厂和存储后端的创建方法改为异步，优化存储初始化流程

### Refactored
- 更新重定向服务，移除布隆过滤器相关代码
- 修改文件存储和 Sled 存储的构造函数为异步
- 统一日志输出格式，提升代码可读性

## [v0.1.5] - 2025-06-05

### Added
- 添加链接查询功能，支持分页和过滤条件
- 添加 admin-panel 目录，包含 Next.js 应用的基础配置和组件
- 添加静态生成参数函数并更新 Next.js 配置
- Implement cookie-based admin login

### Changed
- 优化身份验证和随机代码生成逻辑，使用 OnceLock 缓存环境变量值
- Translate log messages to English

### Fixed
- 更新 cargo.lock 依赖

### Refactored
- 移除未使用的静态生成函数和管理员登录逻辑
- Refactor code structure for improved readability and maintainability

### Style
- 格式化代码，调整导入语句的排版；移除未使用的 debug 导入

### Docs
- document server management commands

## [v0.1.4] - 2025-06-04

### Added
- 添加 CPU_COUNT 环境变量以配置工作线程数量，优化服务器性能
- 添加 colored 库以增强 CLI 输出，移除自定义颜色工具
- 更新 Rust 构建工作流，添加 ARM64 支持，优化交叉编译配置

### Changed
- 更新 .gitignore，添加 shortlinker.pid 和 cargo-flamegraph.trace/，优化文件管理
- 更新 VitePress 工作流，支持手动触发，切换至自托管环境，升级 Node.js 版本
- 更新包描述和作者信息

### Refactored
- 重构多个文件中的 println! 语句，提升代码可读性
- 调整 SQLite 存储配置，优化数据库性能

### Docs
- 更新文档，修正 HTTP 跳转类型为 307，添加数据导入导出命令示例

## [v0.1.3] - 2025-06-03

### Added
- 添加短链接导入导出功能，优化 SQLite 存储配置，大幅度优化编译大小
- sqlite storage 添加 moka TTL 缓存机制
- 添加 APT 包缓存和安装步骤，优化 ARM64 交叉编译器配置
- 为 ARM64 目标添加专用工具链安装，更新 musl 编译器配置
- 更新 Rust 发布工作流以支持 musl 编译，优化交叉编译配置

### Changed
- 更新依赖项，优化文件存储和 SQLite 存储实现，添加日期解析错误处理
- Update ROADMAP.md
- Update rust-release.yml

### Fixed
- 更新 rusqlite 依赖项以启用捆绑特性，添加 cc 依赖
- 修复 UNIX_SOCKET_PATH 环境变量名称，添加 socket 卷到 Dockerfile

## [v0.1.2] - 2025-06-02

### Added
- 添加 Unix socket 支持，优化 HTTP 连接保持策略
- 添加点击量计数器功能到存储模块
- 添加 r2d2 和 r2d2_sqlite 依赖，重构数据库连接管理
- 添加 Docker 部署和相关配置文档

### Changed
- 更新依赖并改进测试和中间件

### Fixed
- UNIX_SOCKET_PATH 在 Windows 下编译的问题
- 更新编辑链接的 GitHub 分支为 master

### Refactored
- 优化短链接存储逻辑

### Docs
- 更新文档以反映新配置选项
- Refactor documentation and improve clarity across multiple files

## [v0.1.1] - 2025-06-01

### Added
- 添加对多种过期时间格式的支持，包括相对时间和 RFC3339 格式

### Changed
- 优化 Rust 编译选项，提升构建性能
- 移除 Dockerfile 中的 links.json 复制步骤

## [v0.1.0] - 2025-06-01

### Added
- 添加 SQLite 存储后端实现，支持短链接的持久化存储
- 添加存储后端名称获取功能
- 添加错误处理机制，重构存储后端以返回自定义错误类型
- Implement CLI for shortlinker with link management and server control

### Changed
- 更新存储配置，支持多种存储后端并修改默认数据库路径
- 更新 Dockerfile 环境变量配置
- 更新短链接服务，重命名处理函数并添加库配置

### Fixed
- 修复 Admin API 可用性日志格式
- 更新 GitHub Actions 配置以获取完整的 Git 历史和所有标签
- 添加交叉编译工具和配置以支持多种目标平台

### Refactored
- 重构存储模块并修复 dotenv 加载问题
- 重构存储后端，支持依赖注入并更新相关代码
- 重构 CLI 相关模块，优化帮助信息和链接管理，添加锁文件管理功能
- 重构服务模块，添加管理和重定向服务
- 公开解析器方法并重构测试结构
- 删除测试文件和基准测试，清理代码库

### Style
- 优化代码结构和格式，调整导入顺序，清理多余空行，增强可读性

### Docs
- 统一存储后端配置文档
- 统一存储配置并添加全面测试覆盖
- 添加 Cloudflare Worker 的英文和中文文档
- 添加文档以支持短链接管理面板的使用说明
- 添加英文文档支持并完善文档内容
- 添加多存储后端支持文档

## [v0.0.6] - 2025-05-30

### Added
- 更新短链接存储实现，添加示例链接返回功能
- 添加讨论、创意、问答和展示模板，增强社区互动和反馈机制
- 添加贡献指南文档，提供项目贡献流程和规范
- 优化 changelog 生成逻辑，支持根据提交类型分类并添加比较链接
- 优化 Rust 发布工作流中的徽章显示格式

### Changed
- 更新 README 文件，添加徽章以增强可视化信息
- 更新 docs 依赖项版本

### Fixed
- 修改鉴权函数以返回简化的 404 响应体
- 更新获取标签逻辑以支持当前标签和上一个标签之间的提交信息
- 更新 GitHub 容器注册表地址以指向正确的仓库
- 更新 Dockerfile 以支持 musl 目标的静态链接编译
- 更新 README 文件中的 Docker 镜像版本和 GitHub 工作流状态徽章
- 处理 Docker 容器重启时的 PID 文件清理逻辑
- 移除不必要的 RwLock 导入，简化存储模块
- 更新文档中的分支名称，从 main 修改为 master

### Style
- 在 print_usage 宏中添加分号

### Refactored
- 重构存储系统实现

## [v0.0.5] - 2025-05-29

### Added
- 添加 Admin API 支持，提供短链接管理的 HTTP 接口
- 添加 VitePress 文档部署工作流
- add VitePress documentation site
- 更新 Admin API 鉴权逻辑，禁用时返回 404，添加相关文档说明
- 增强鉴权函数，添加未设置管理员令牌的日志信息
- 更新 admin 路由前缀配置，支持通过环境变量设置

### Changed
- 更新文档，添加链接过期时间支持和环境变量配置示例
- 更新短链接处理逻辑，优化 HTTP 响应头和日志记录
- 更新 Docker 构建流程，优化文件复制路径和构建命令
- 重构构建流程，合并 Linux 和 Windows 构建作业，添加 macOS 构建支持
- 更新发布流程，优化二进制文件下载链接和文档

## [v0.0.4] - 2025-05-28

### Added
- 添加文件存储和 Redis 存储支持，重构 CLI 功能
- 添加链接过期检查，更新短链接重定向逻辑
- 重构 CLI 输出，添加彩色输出宏以增强用户体验
- 添加对 Windows 系统的锁文件支持，优化进程管理

### Changed
- 更新 Windows 平台的锁文件机制，优化链接配置重载逻辑
- 更新 Docker 和 Rust CI 工作流，添加文件路径监控以支持更改检测
- 更新发布说明，简化内容并添加文档链接
- 更新依赖项

## [v0.0.3] - 2025-05-28

### Added
- 添加 PID 文件管理功能，防止服务器重复启动 (fix #1)
- 添加短链接存储和加载功能，重构信号通知机制
- 添加短链 worker 模块并改进响应处理
- 添加变更日志生成步骤，更新发布说明文档
- 添加 workflow_dispatch 触发器，优化 Docker CI 工作流

### Changed
- 优化随机短码生成
- 更新 Rust CI 工作流，添加 i686 目标支持并优化发布信息
- 更新 Dockerfile，修复 links.json 复制路径并简化构建命令
- Create CNAME

## [v0.0.2] - 2025-05-27

### Added
- 添加随机短码生成机制，支持短码覆盖选项
- 添加 Docker 支持，创建 Dockerfile 和相关配置文件
- 添加 Unix 和 Windows 平台的信号与文件监听机制以支持动态重新加载配置
- 添加 Docker CI 工作流，支持自动构建和推送 Docker 镜像

### Changed
- 更新 README，增强项目亮点和使用说明，添加 Docker 部署示例
- 更新 Docker CI 工作流，修正镜像名称为短链接服务
- 优化交叉编译逻辑

## [v0.0.1] - 2025-05-27

### Added
- 实现基本短链接服务
- 更新短链接服务，添加链接管理功能和 PID 文件支持

## [v0.0.0] - 2025-05-26

### Added
- 初始化 shortlinker rust 项目
- 设置 CI/CD 和交叉编译
- 添加权限配置并优化发布目录创建步骤
- Update README.md
- Initial commit

[Unreleased]: https://github.com/AptS-1547/shortlinker/compare/v0.5.0-beta.1...HEAD
[v0.5.0-beta.1]: https://github.com/AptS-1547/shortlinker/compare/v0.5.0-alpha.6...v0.5.0-beta.1
[v0.5.0-alpha.6]: https://github.com/AptS-1547/shortlinker/compare/v0.5.0-alpha.5...v0.5.0-alpha.6
[v0.5.0-alpha.5]: https://github.com/AptS-1547/shortlinker/compare/v0.5.0-alpha.4...v0.5.0-alpha.5
[v0.5.0-alpha.4]: https://github.com/AptS-1547/shortlinker/compare/v0.5.0-alpha.3...v0.5.0-alpha.4
[v0.5.0-alpha.3]: https://github.com/AptS-1547/shortlinker/compare/v0.5.0-alpha.2...v0.5.0-alpha.3
[v0.5.0-alpha.2]: https://github.com/AptS-1547/shortlinker/compare/v0.5.0-alpha.1...v0.5.0-alpha.2
[v0.5.0-alpha.1]: https://github.com/AptS-1547/shortlinker/compare/v0.4.3...v0.5.0-alpha.1
[v0.4.3]: https://github.com/AptS-1547/shortlinker/compare/v0.4.2...v0.4.3
[v0.4.2]: https://github.com/AptS-1547/shortlinker/compare/v0.4.1...v0.4.2
[v0.4.1]: https://github.com/AptS-1547/shortlinker/compare/v0.4.0...v0.4.1
[v0.4.0]: https://github.com/AptS-1547/shortlinker/compare/v0.3.0...v0.4.0
[v0.3.0]: https://github.com/AptS-1547/shortlinker/compare/v0.2.2...v0.3.0
[v0.3.0-beta.3]: https://github.com/AptS-1547/shortlinker/compare/v0.3.0-beta.2...v0.3.0-beta.3
[v0.3.0-beta.2]: https://github.com/AptS-1547/shortlinker/compare/v0.3.0-beta.1...v0.3.0-beta.2
[v0.3.0-beta.1]: https://github.com/AptS-1547/shortlinker/compare/v0.3.0-alpha.7...v0.3.0-beta.1
[v0.3.0-alpha.7]: https://github.com/AptS-1547/shortlinker/compare/v0.3.0-alpha.6...v0.3.0-alpha.7
[v0.3.0-alpha.6]: https://github.com/AptS-1547/shortlinker/compare/v0.3.0-alpha.5...v0.3.0-alpha.6
[v0.3.0-alpha.5]: https://github.com/AptS-1547/shortlinker/compare/v0.3.0-alpha.4...v0.3.0-alpha.5
[v0.3.0-alpha.4]: https://github.com/AptS-1547/shortlinker/compare/v0.3.0-alpha.3...v0.3.0-alpha.4
[v0.3.0-alpha.3]: https://github.com/AptS-1547/shortlinker/compare/v0.3.0-alpha.2...v0.3.0-alpha.3
[v0.3.0-alpha.2]: https://github.com/AptS-1547/shortlinker/compare/v0.3.0-alpha.1...v0.3.0-alpha.2
[v0.3.0-alpha.1]: https://github.com/AptS-1547/shortlinker/compare/v0.2.3-alpha.3...v0.3.0-alpha.1
[v0.2.3-alpha.3]: https://github.com/AptS-1547/shortlinker/compare/v0.2.3-alpha.2...v0.2.3-alpha.3
[v0.2.3-alpha.2]: https://github.com/AptS-1547/shortlinker/compare/v0.2.3-alpha.1...v0.2.3-alpha.2
[v0.2.3-alpha.1]: https://github.com/AptS-1547/shortlinker/compare/v0.2.2...v0.2.3-alpha.1
[v0.2.2]: https://github.com/AptS-1547/shortlinker/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/AptS-1547/shortlinker/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.7...v0.2.0
[v0.1.7-alpha.7]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.6...v0.1.7-alpha.7
[v0.1.7-alpha.6]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.5...v0.1.7-alpha.6
[v0.1.7-alpha.5]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.4...v0.1.7-alpha.5
[v0.1.7-alpha.4]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.3...v0.1.7-alpha.4
[v0.1.7-alpha.3]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.2...v0.1.7-alpha.3
[v0.1.7-alpha.2]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.1...v0.1.7-alpha.2
[v0.1.7-alpha.1]: https://github.com/AptS-1547/shortlinker/compare/v0.1.6...v0.1.7-alpha.1
[v0.1.6]: https://github.com/AptS-1547/shortlinker/compare/v0.1.5...v0.1.6
[v0.1.5]: https://github.com/AptS-1547/shortlinker/compare/v0.1.4...v0.1.5
[v0.1.4]: https://github.com/AptS-1547/shortlinker/compare/v0.1.3...v0.1.4
[v0.1.3]: https://github.com/AptS-1547/shortlinker/compare/v0.1.2...v0.1.3
[v0.1.2]: https://github.com/AptS-1547/shortlinker/compare/v0.1.1...v0.1.2
[v0.1.1]: https://github.com/AptS-1547/shortlinker/compare/v0.1.0...v0.1.1
[v0.1.0]: https://github.com/AptS-1547/shortlinker/compare/v0.0.6...v0.1.0
[v0.0.6]: https://github.com/AptS-1547/shortlinker/compare/v0.0.5...v0.0.6
[v0.0.5]: https://github.com/AptS-1547/shortlinker/compare/v0.0.4...v0.0.5
[v0.0.4]: https://github.com/AptS-1547/shortlinker/compare/v0.0.3...v0.0.4
[v0.0.3]: https://github.com/AptS-1547/shortlinker/compare/v0.0.2...v0.0.3
[v0.0.2]: https://github.com/AptS-1547/shortlinker/compare/v0.0.1...v0.0.2
[v0.0.1]: https://github.com/AptS-1547/shortlinker/compare/v0.0.0...v0.0.1
[v0.0.0]: https://github.com/AptS-1547/shortlinker/releases/tag/v0.0.0
