# shortlinker

一个极简主义的短链接服务，支持 HTTP 302 跳转，使用 Rust 编写，部署便捷、响应快速，适用于自建短链系统。

## ✨ 项目亮点

- 🚀 **高性能**：基于 Rust 构建，速度与安全性并存。
- 🔗 **302 跳转**：临时性重定向，适用于点击追踪、平台导流等场景。
- 🎯 **自定义路径**：支持用户自定义短链，例如 `esap.cc/github`
- 🎲 **随机路径生成**（开发中）：可为目标链接自动生成短链，如 `esap.cc/aB3dF1`

## 示例使用（绑定域名）

本项目推荐通过自有域名部署，例如绑定 `esap.cc`，用户可访问以下形式的短链：

- `https://esap.cc/github` → 跳转至 GitHub
- `https://esap.cc/abc123` → 跳转至其他目标链接

## 快速开始

```bash
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker
cargo run
```

默认监听在 127.0.0.1:8080，你可以通过配置反向代理（如 Caddy/Nginx）绑定域名并启用 HTTPS（推荐）。

## 映射配置

当前映射关系写在代码中：

```rust
// 示例：src/routes.rs
let redirect_map = HashMap::from([
    ("github", "https://github.com"),
    ("gh", "https://github.com/AptS-1547"),
]);
```

你可以自由添加或读取外部配置文件（如 JSON、YAML、TOML），后续将考虑支持动态添加与管理。

## 部署示例：Caddy（推荐）

使用 Caddy 自动启用 HTTPS，并绑定你的域名：

```caddy
esap.cc {
    reverse_proxy 127.0.0.1:8080
}
```

部署后即可使用 https://esap.cc/github 格式访问跳转。

## 项目结构

```
shortlinker/
├── Cargo.toml
└── src/
    ├── main.rs        // 服务启动入口
    └── routes.rs      // 路由逻辑与跳转映射
```

## 开发中功能

- 随机路径生成（如 esap.cc/aB3dF1）
- 持久化存储映射关系（文件/数据库）
- 管理面板（查看/添加/删除跳转规则）
- API 支持（短链生成、点击统计）

📜 License

MIT License © AptS:1547
