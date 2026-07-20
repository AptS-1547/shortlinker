# CLI 概览

Shortlinker 提供了直观易用的命令行工具，用于管理短链接。

## 工具特性

- 🎨 **彩色输出** - 清晰的视觉反馈
- 🔄 **实时同步** - 命令执行立即生效  
- ⚡ **快速响应** - 支持 SQLite、PostgreSQL、MySQL、MariaDB 等数据库存储后端
- 🛡️ **错误处理** - 详细的错误信息和建议
- 📦 **数据导入导出** - 链接导入/导出仅支持 CSV 格式

## 基本语法

```bash
./shortlinker                          # 启动 HTTP 服务器（默认）
./shortlinker <command> [args] [opts]  # 运行 CLI 命令
./shortlinker --socket <路径> <command> # 覆盖 IPC socket 路径（Unix）/命名管道（Windows）
```

## 命令概览

> 注意：`start/stop/restart` 不是内置子命令；服务启动直接运行 `./shortlinker`，停止/重启通常由 systemd/Docker 或发送信号完成（见部署/运维文档）。

| 命令 | 功能 | 示例 |
|------|------|------|
| `help` | 查看帮助 | `./shortlinker help` |
| `add` | 添加短链接 | `./shortlinker add github https://github.com` |
| `remove` | 删除短链接 | `./shortlinker remove github` |
| `update` | 更新短链接 | `./shortlinker update github https://new-url.com` |
| `list` | 列出所有链接 | `./shortlinker list` |
| `export` | 导出数据 | `./shortlinker export backup.csv` |
| `import` | 导入数据 | `./shortlinker import backup.csv --force` |
| `status` | 查看服务状态（IPC） | `./shortlinker status` |
| `config generate` | 生成配置模板 | `./shortlinker config generate` |
| `reset-password` | 重置管理员密码 | `./shortlinker reset-password` |
| `config` | 运行时配置管理（数据库） | `./shortlinker config list` |

## 快速示例

### 基础操作
```bash
# 添加短链接
./shortlinker add docs https://docs.example.com

# 查看所有链接
./shortlinker list

# 删除链接
./shortlinker remove docs
```

### 数据管理
```bash
# 导出数据
./shortlinker export backup.csv

# 导入数据
./shortlinker import backup.csv --force
```

### 高级功能
```bash
# 随机短码
./shortlinker add https://example.com

# 设置过期时间
./shortlinker add sale https://shop.com/sale --expire 1d

# 强制覆盖
./shortlinker add docs https://new-docs.com --force

```

## 下一步

- 📖 查看 [详细命令参考](/cli/commands) 了解所有选项
- ⚙️ 学习 [配置指南](/config/) 自定义行为
