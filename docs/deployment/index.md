# 部署指南

Shortlinker 支持多种部署方式，从简单的本地运行到生产环境的容器化部署。

## 推荐阅读顺序

1. [Docker 部署概览](/deployment/docker)
2. [Docker 快速开始与 Compose](/deployment/docker-quickstart)
3. [反向代理概览](/deployment/proxy)
4. [systemd 服务概览](/deployment/systemd)

如果需要生产运维细节，再继续阅读：

- [Docker 运维与安全](/deployment/docker-operations)
- [反向代理性能优化与监控](/deployment/proxy-operations)
- [systemd Docker Compose 与运维](/deployment/systemd-operations)

## 部署方式概览

| 方式 | 适用场景 | 推荐程度 |
|------|----------|----------|
| Docker | 大多数生产环境 | ⭐⭐⭐⭐⭐ |
| 预编译二进制 | 快速本地验证 / 轻量部署 | ⭐⭐⭐⭐ |
| 源码编译 | 需要自定义构建特性 | ⭐⭐⭐ |

## 前置准备

- **操作系统**：Linux、macOS、Windows
- **架构**：x86_64、ARM64
- **源码编译额外要求**：Rust `>= 1.88.0`（Edition 2024）、Git

## 部署架构

```
用户请求 → 反向代理 → Shortlinker 服务 → 数据存储
    ↓           ↓              ↓           ↓
  浏览器      Nginx         Docker      SQLite(默认)
  curl        Caddy         systemd     MySQL/PostgreSQL
  API         Apache        Binary      MariaDB
```

## 安全建议

1. **网络安全**：通过反向代理暴露服务
2. **文件权限**：合理设置数据文件权限
3. **进程管理**：使用系统服务管理器
4. **数据备份**：定期备份链接数据（SQLite 可直接备份 .db 文件）

## 性能特征

- **响应时间**: < 1ms（SQLite 本地存储）
- **并发支持**: 数千个并发连接
- **内存使用**: 极低内存占用
- **存储格式**: SQLite 数据库（默认），支持 MySQL、PostgreSQL、MariaDB

## 下一步

- 📦 [Docker 部署概览](/deployment/docker)
- ⚡ [Docker 快速开始与 Compose](/deployment/docker-quickstart)
- 🛠️ [Docker 运维与安全](/deployment/docker-operations)
- 🔀 [反向代理概览](/deployment/proxy)
- 📈 [反向代理性能优化与监控](/deployment/proxy-operations)
- ⚙️ [systemd 服务概览](/deployment/systemd)
- 🔧 [systemd Docker Compose 与运维](/deployment/systemd-operations)

需要配置帮助？查看 [配置指南](/config/) 了解 `config.toml`（启动配置）与数据库运行时配置的设置方式。
