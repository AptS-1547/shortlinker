# 安装指南

选择适合您的安装方式快速部署 Shortlinker。

## 环境要求

### 运行环境

- 操作系统：Linux、macOS、Windows
- 网络连接：用于下载依赖

### 源码编译环境

- **Rust**: >= 1.88.0 (必需，Edition 2024)
- **Git**: 用于克隆项目

## 安装方式

### 🐳 Docker 部署（推荐）

无需任何依赖，一条命令启动：

```bash
# 基础运行
docker run -d -p 8080:8080 e1saps/shortlinker

# 数据持久化（推荐）
docker run -d -p 8080:8080 -v $(pwd)/data:/data e1saps/shortlinker
```

### 📦 预编译二进制

下载对应平台的预编译版本：

```bash
# Linux x64
wget https://github.com/AptS-1547/shortlinker/releases/latest/download/shortlinker-linux-x64.tar.gz
tar -xzf shortlinker-linux-x64.tar.gz
./shortlinker

# macOS
wget https://github.com/AptS-1547/shortlinker/releases/latest/download/shortlinker-macos.tar.gz

# Windows
# 下载 shortlinker-windows.zip 并解压
```

### 🔧 源码编译

适合需要定制的用户：

```bash
# 1. 安装 Rust (如果没有)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. 检查版本
rustc --version  # 应该 >= 1.88.0

# 3. 克隆并编译
git clone https://github.com/AptS-1547/shortlinker.git
cd shortlinker

# 默认编译（server + CLI）
cargo build --release

# 仅服务器（不包含 CLI）
cargo build --release --no-default-features --features server

# 启用 Prometheus metrics（导出到 /health/metrics，需要编译时启用）
cargo build --release --features metrics

# 全功能编译（服务器 + CLI + Metrics）
cargo build --release --features full

# 4. 运行
./target/release/shortlinker
```

## 快速验证

安装完成后，验证服务是否正常：

```bash
# 启动服务
./shortlinker

# 另开终端测试
curl -I http://localhost:8080/
# 应该返回 307 重定向
```

## 常见问题

### Rust 版本过低

```bash
# 更新到最新版本
rustup update
```

### 编译失败

```bash
# 清理后重新编译
cargo clean && cargo build --release
```

### 端口被占用

```bash
# 使用其他端口：在 config.toml 中设置
# [server]
# port = 3000
./shortlinker
```

## 下一步

安装完成后，继续阅读：

- 🚀 [快速开始](/guide/getting-started) - 学习基本使用
- ⚙️ [配置指南](/config/) - 了解配置选项
- 📋 [CLI 工具](/cli/) - 掌握命令行操作
