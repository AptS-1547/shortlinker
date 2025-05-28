# 部署指南

Shortlinker 支持多种部署方式，从简单的本地运行到生产环境的容器化部署。

## 部署方式概览

### 🚀 快速部署
- **Docker 部署**：推荐的生产环境方案，无需安装 Rust
- **预编译二进制**：下载即用，支持多平台
- **源码编译**：需要 Rust 1.82+，适合定制需求

### 🔧 生产环境
- **反向代理**：Nginx、Caddy、Apache 配置
- **系统服务**：systemd、Docker Compose 管理
- **监控告警**：健康检查和日志管理

## 环境要求

### 系统要求
- **操作系统**: Linux、macOS、Windows
- **架构**: x86_64、ARM64

### 源码编译要求
- **Rust**: >= 1.82.0 (必需)
- **Git**: 用于克隆项目

## 快速开始

### Docker 部署（推荐）
```bash
# 快速启动
docker run -d -p 8080:8080 -v $(pwd)/data:/data e1saps/shortlinker
```

### 预编译二进制
```bash
# 下载并运行
wget https://github.com/AptS-1547/shortlinker/releases/latest/download/shortlinker-linux-x64.tar.gz
tar -xzf shortlinker-linux-x64.tar.gz
./shortlinker
```

### 源码编译
```bash
# 克隆并编译
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker
cargo build --release
./target/release/shortlinker
```

## 部署架构

```
用户请求 → 反向代理 → Shortlinker 服务 → 数据存储
    ↓           ↓              ↓           ↓
  浏览器      Nginx         Docker      JSON文件
  curl        Caddy         systemd     
  API         Apache        Binary
```

## 安全建议

1. **网络安全**：通过反向代理暴露服务
2. **文件权限**：合理设置数据文件权限
3. **进程管理**：使用系统服务管理器
4. **数据备份**：定期备份链接数据

## 性能特征

- **响应时间**: < 1ms（本地存储）
- **并发支持**: 数千个并发连接
- **内存使用**: 极低内存占用
- **存储格式**: JSON 文件，支持热重载

## 下一步

选择适合您的部署方式：

- 📦 [Docker 部署](/deployment/docker) - 容器化部署详细指南
- 🔀 [反向代理](/deployment/proxy) - Nginx、Caddy 配置
- ⚙️ [系统服务](/deployment/systemd) - systemd 和进程管理

需要配置帮助？查看 [配置说明](/config/) 了解环境变量设置。
