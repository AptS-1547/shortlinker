# 配置指南

为了避免单页过长，配置文档已拆分为“概览 + 专题页”。

## 文档导航

- [启动配置参数](/config/startup)
- [运行时配置参数](/config/runtime)
- [安全最佳实践](/config/security)
- [配置示例与热重载](/config/examples)
- [存储后端配置](/config/storage)
- [存储后端详解](/config/storage-backends)
- [存储选型与性能](/config/storage-selection)
- [存储迁移与运维](/config/storage-operations)

## 配置架构

Shortlinker 的配置分为两类：

- **启动配置**：存储在 `config.toml` 文件中，修改后需要重启服务
- **动态配置**：存储在数据库中，可通过管理面板在运行时修改

```text
config.toml (启动时读取)
       ↓
StaticConfig (启动配置，内存)
       ↓
   数据库 (短链接数据 + 运行时配置)
       ↓
RuntimeConfig (运行时配置缓存，内存)
       ↓
    业务逻辑（路由/鉴权/缓存等）
```

首次启动时，服务会根据代码内置的配置定义把**运行时配置默认值**初始化到数据库，并加载到内存缓存；之后以数据库中的值为准。  
当前版本不会从 `config.toml` 或环境变量“迁移/覆盖”运行时配置。

## 配置方式速览

- 启动配置：通过 `config.toml` + `SL__...` 环境变量（见 [启动配置参数](/config/startup)）
- 运行时配置：通过 Admin API / 管理面板（见 [运行时配置参数](/config/runtime)）
- 存储配置：`database.database_url` 及各数据库 URL（见 [存储后端配置](/config/storage)）

## 配置优先级

1. **数据库（运行时配置）**：`api.*` / `routes.*` / `features.*` / `click.*` / `cors.*` / `analytics.*`
2. **环境变量（启动配置覆盖）**：`SL__...`
3. **`config.toml`（启动配置）**：`[server]` / `[database]` / `[cache]` / `[logging]` / `[analytics]` / `[ipc]`
4. **程序默认值**

> 说明：环境变量只影响**启动配置**；当前版本不会自动把环境变量或 `config.toml` 迁移到运行时配置。

## 下一步

- 📋 查看 [存储后端配置](/config/storage)
- 🚀 学习 [部署配置](/deployment/) 生产环境设置
- 🛡️ 了解 [Admin API](/api/admin) 管理接口使用
- 🏥 了解 [健康检查 API](/api/health) 监控接口使用
