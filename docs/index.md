---
layout: home

hero:
  name: "Shortlinker"
  text: "极简主义短链接服务"
  tagline: "支持 HTTP 307 跳转，使用 Rust 编写，部署便捷、响应快速"
  image:
    src: /logo.svg
    alt: Shortlinker
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/getting-started
    - theme: alt
      text: 查看 GitHub
      link: https://github.com/AptS-1547/shortlinker

features:
  - icon: 🚀
    title: 高性能
    details: 基于 Rust 构建，毫秒级重定向响应，支持高并发访问
  - icon: 💾
    title: 多存储后端
    details: 支持 SQLite（默认）、PostgreSQL、MySQL、MariaDB 等数据库存储方案
  - icon: 🛡️
    title: 安全可靠
    details: Admin API 鉴权、健康检查监控、进程保护机制
  - icon: 🐳
    title: 部署简单
    details: Docker 一键部署，支持 systemd 服务管理
  - icon: ⚡
    title: 热重载
    details: 运行时添加/删除短链，无需重启服务器
  - icon: 🎯
    title: 智能管理
    details: 自定义短码、随机生成、过期时间、CLI 工具管理和 TUI 界面
---

:::warning ⚠️ v0.3.x 版本提醒
当前版本 (v0.3.x) 正在进行**大幅度功能调整和重构**，更新频率较高，可能会有 API 变更或功能调整。

**建议**：
- 📌 **生产环境**：使用稳定版本标签（如 `v0.2.x`）
- 🔄 **开发环境**：跟随最新版本体验新功能
- 📖 **文档参考**：文档可能滞后于代码实现，以实际功能为准
- 🐛 **问题反馈**：欢迎通过 [GitHub Issues](https://github.com/AptS-1547/shortlinker/issues) 报告问题
:::

## 设计理念

### 极简主义
专注于短链接重定向核心功能，配置简单，部署快速

### 高性能
Rust 原生性能保障，SQLite 提供生产级数据库性能，异步并发处理

### 易于使用
命令行工具管理，环境变量配置，Docker 一键部署

## 核心特性

- **多存储后端**：SQLite 数据库（默认）、PostgreSQL、MySQL、MariaDB 等数据库存储
- **Admin API**：HTTP API 管理接口，支持鉴权和自定义路由前缀
- **健康监控**：完整的健康检查 API，支持存储状态和运行时间监控
- **智能过期**：支持灵活的时间格式设置，自动失效和清理
- **跨平台支持**：Windows、Linux、macOS，智能进程锁防止重复启动
- **容器优化**：Docker 镜像部署，支持容器重启检测
- **TUI 界面**：终端用户界面，支持交互式链接管理
- **TOML 配置**：灵活的配置文件系统，支持环境变量覆盖

## 快速体验

```bash
# Docker 一键启动
docker run -d -p 8080:8080 e1saps/shortlinker

# CLI 添加短链接
./shortlinker add github https://github.com

# TUI 界面管理
./shortlinker tui

# 访问短链接
curl -L http://localhost:8080/github
```

## 使用场景

- **营销推广**：活动链接、社交媒体分享
- **内部工具**：文档跳转、系统集成
- **临时链接**：限时分享、测试环境
- **API 服务**：微服务间链接管理


## 开始使用

准备好了吗？查看 [快速开始指南](/guide/getting-started) 开始您的 Shortlinker 之旅。

更多组件：[Web 管理界面](/admin-panel/) | [Cloudflare Worker 版本](/cf-worker/)。
