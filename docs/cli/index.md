# CLI 命令行工具

Shortlinker 提供了直观易用的命令行工具，用于管理短链接。

## 工具特性

- 🎨 **彩色输出** - 清晰的视觉反馈
- 🔄 **实时同步** - 命令执行立即生效  
- ⚡ **快速响应** - 支持 SQLite、PostgreSQL、MySQL、MariaDB 等数据库存储后端
- 🛡️ **错误处理** - 详细的错误信息和建议
- 📦 **数据导入导出** - JSON 格式备份和迁移支持

## 基本语法

```bash
./shortlinker <command> [arguments] [options]
```

## 命令概览

| 命令 | 功能 | 示例 |
|------|------|------|
| `help` | 查看帮助 | `./shortlinker help` |
| `start` | 启动服务器 | `./shortlinker start` |
| `stop` | 停止服务器 | `./shortlinker stop` |
| `restart` | 重启服务器 | `./shortlinker restart` |
| `add` | 添加短链接 | `./shortlinker add github https://github.com` |
| `remove` | 删除短链接 | `./shortlinker remove github` |
| `update` | 更新短链接 | `./shortlinker update github https://new-url.com` |
| `list` | 列出所有链接 | `./shortlinker list` |
| `export` | 导出数据 | `./shortlinker export backup.json` |
| `import` | 导入数据 | `./shortlinker import backup.json --force` |
| `tui` | 启动 TUI 界面 | `./shortlinker tui` |

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
./shortlinker export backup.json

# 导入数据
./shortlinker import backup.json --force
```

### 高级功能
```bash
# 随机短码
./shortlinker add https://example.com

# 设置过期时间
./shortlinker add sale https://shop.com/sale --expire 1d

# 强制覆盖
./shortlinker add docs https://new-docs.com --force

# 启动 TUI 界面
./shortlinker tui
```

## 下一步

- 📖 查看 [详细命令参考](/cli/commands) 了解所有选项
- ⚙️ 学习 [配置说明](/config/) 自定义行为
