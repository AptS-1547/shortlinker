---
layout: home

hero:
  name: "Shortlinker"
  text: "极简主义短链接服务"
  tagline: "支持 HTTP 302 跳转，使用 Rust 编写，部署便捷、响应快速"
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
    details: 基于 Rust + Actix-web 构建，提供毫秒级的重定向响应速度
  - icon: 🎯
    title: 动态管理
    details: 支持运行时添加/删除短链，无需重启服务器
  - icon: 🎲
    title: 智能短码
    details: 支持自定义短码和随机生成，避免冲突的智能处理
  - icon: ⏰
    title: 过期时间
    details: 支持设置链接过期时间，自动失效和清理
  - icon: 💾
    title: 持久化存储
    details: JSON 文件存储，支持热重载和配置文件自动检测
  - icon: 🔄
    title: 跨平台
    details: 支持 Windows、Linux、macOS，智能进程锁防止重复启动
  - icon: 🐳
    title: 容器化
    details: 优化的 Docker 镜像部署，多阶段构建，scratch 基础镜像
  - icon: 🔐
    title: 进程管理
    details: 智能进程保护，PID 文件管理，防止重复启动实例
---
## 为什么选择 Shortlinker

### 💡 极简设计

专注于核心功能，无多余特性，配置简单，部署快速

### ⚡ 性能优越

Rust 原生性能，毫秒级响应，支持高并发访问

### 🛠️ 运维友好

单一二进制，Docker 支持，systemd 集成，监控完备

## 快速体验

### 三种部署方式

```bash
# 1. Docker 一键启动
docker run -d -p 8080:8080 e1saps/shortlinker

# 2. 下载预编译版本
wget https://github.com/AptS-1547/shortlinker/releases/latest/download/shortlinker-linux-x86_64

# 3. 源码编译
git clone https://github.com/AptS-1547/shortlinker && cd shortlinker && cargo run
```

### 基本使用

```bash
# 添加短链接
./shortlinker add github https://github.com

# 访问短链接
curl -L http://localhost:8080/github
```

## 开始使用

准备好了吗？查看 [快速开始指南](/guide/getting-started) 开始您的 Shortlinker 之旅！
