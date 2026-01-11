# shortlinker

<div align="center">

[![GitHub 最新发布](https://img.shields.io/github/v/release/AptS-1547/shortlinker)](https://github.com/AptS-1547/shortlinker/releases)
[![Rust 构建状态](https://img.shields.io/github/actions/workflow/status/AptS-1547/shortlinker/rust-release.yml?label=rust%20release)](https://github.com/AptS-1547/shortlinker/actions/workflows/rust-release.yml)
[![Docker 构建状态](https://img.shields.io/github/actions/workflow/status/AptS-1547/shortlinker/docker-image.yml?label=docker%20build)](https://github.com/AptS-1547/shortlinker/actions/workflows/docker-image.yml)
[![CodeFactor 评分](https://www.codefactor.io/repository/github/apts-1547/shortlinker/badge)](https://www.codefactor.io/repository/github/apts-1547/shortlinker)
[![MIT 协议](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker 拉取数](https://img.shields.io/docker/pulls/e1saps/shortlinker)](https://hub.docker.com/r/e1saps/shortlinker)

一个支持 HTTP 307 重定向的极简短链接服务，使用 Rust 构建。

[English](README.md) | [中文](README.zh.md)

![管理面板](assets/admin-panel-dashboard.jpeg)

</div>

## 功能特性

- 基于 Rust + Actix-web 的高性能服务
- 多种存储后端：SQLite、MySQL、PostgreSQL
- 运行时动态管理链接，无需重启
- 支持自定义和随机短码
- 灵活的过期时间格式
- 链接密码保护
- Bearer Token 认证的管理 API
- Web 管理面板
- 终端 TUI 模式
- Docker 和 Unix Socket 支持

## 快速开始

**Docker:**

```bash
docker run -d -p 8080:8080 -v $(pwd)/data:/data e1saps/shortlinker
```

**本地运行:**

```bash
git clone https://github.com/AptS-1547/shortlinker && cd shortlinker
cargo run
```

## CLI 用法

```bash
./shortlinker                                    # 启动服务器
./shortlinker tui                                # TUI 模式（需要 'tui' feature）
./shortlinker add github https://github.com     # 自定义短码
./shortlinker add https://example.com           # 随机短码
./shortlinker add secret https://example.com --password mypass  # 密码保护
./shortlinker add temp https://example.com --expire 7d          # 7 天后过期
./shortlinker list                              # 列出所有链接
./shortlinker remove github                     # 删除链接
./shortlinker export links.json                 # 导出到 JSON
./shortlinker import links.json                 # 从 JSON 导入
```

## 管理 API

```bash
# 设置 token
export ADMIN_TOKEN=your_secret_token

# 获取所有链接
curl -H "Authorization: Bearer $ADMIN_TOKEN" http://localhost:8080/admin/link

# 创建链接
curl -X POST -H "Authorization: Bearer $ADMIN_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com","expires_at":"7d"}' \
     http://localhost:8080/admin/link

# 删除链接
curl -X DELETE -H "Authorization: Bearer $ADMIN_TOKEN" \
     http://localhost:8080/admin/link/github
```

批量操作、运行时配置等详见 [管理 API 文档](docs/api/admin.md)。

## 配置

生成配置文件：

```bash
./shortlinker generate-config
```

这会创建 `config.toml`，包含服务器、数据库、缓存和日志设置。

详细配置选项见 [配置文档](docs/config/index.md)。

## 文档

- [快速入门](docs/guide/getting-started.md)
- [配置说明](docs/config/index.md)
- [存储后端](docs/config/storage.md)
- [管理 API](docs/api/admin.md)
- [健康检查 API](docs/api/health.md)
- [Docker 部署](docs/deployment/docker.md)
- [systemd 服务](docs/deployment/systemd.md)
- [CLI 命令](docs/cli/commands.md)

## 相关项目

- [Web 管理面板](admin-panel/) - 图形化链接管理
- [Cloudflare Worker](cf-worker/) - Serverless 版本

## 许可证

MIT License © AptS:1547

<pre>
        ／＞　 フ
       | 　_　_|    AptS:1547
     ／` ミ＿xノ    — shortlinker assistant bot —
    /　　　　 |
   /　 ヽ　　 ﾉ      Rust / SQLite / Bloom / CLI
   │　　|　|　|
／￣|　　 |　|　|
(￣ヽ＿_ヽ_)__)
＼二)

   「ready to 307 !」
</pre>
