# CLI 命令参考

详细的命令行工具使用说明和参数选项。

## 基本命令

### add - 添加短链接

```bash
./shortlinker add <短码> <目标URL> [选项]
./shortlinker add <目标URL> [选项]  # 随机短码
```

**选项**:
- `--force`: 强制覆盖已存在的短码
- `--expire <时间>`: 设置过期时间

**示例**:
```bash
./shortlinker add google https://www.google.com
./shortlinker add https://www.example.com  # 随机短码
./shortlinker add daily https://example.com --expire 1d
./shortlinker add google https://www.google.com --force
```

### export - 导出短链接

```bash
./shortlinker export [文件路径]
```

**示例**:
```bash
./shortlinker export  # 默认文件名
./shortlinker export backup.json
```

### import - 导入短链接

```bash
./shortlinker import <文件路径> [选项]
```

**选项**:
- `--force`: 强制覆盖已存在的短码

**示例**:
```bash
./shortlinker import backup.json
./shortlinker import backup.json --force
```

### remove - 删除短链接

```bash
./shortlinker remove <短码>
```

### list - 列出短链接

```bash
./shortlinker list
```

### help - 查看帮助

```bash
./shortlinker help
```

### tui - 启动终端用户界面

```bash
./shortlinker tui
```

**TUI 模式特点**：
- 交互式可视化界面
- 实时查看所有短链接列表
- 支持键盘导航和操作
- 显示链接详细信息（点击数、过期时间等）

**快捷键**：
- `↑/↓` 或 `j/k`：上下移动选择
- `Enter`：查看详情
- `q` 或 `Esc`：退出
- `r`：刷新列表

> 💡 **提示**：TUI 模式适合快速浏览和管理链接，详细使用说明请参考 [TUI 使用指南](/cli/tui)

### update - 更新短链接

```bash
./shortlinker update <短码> <新目标URL> [选项]
```

**选项**:
- `--expire <时间>`: 设置新的过期时间
- `--password <密码>`: 设置或更新密码

**示例**:
```bash
./shortlinker update github https://new-github.com
./shortlinker update github https://new-github.com --expire 30d
./shortlinker update github https://new-github.com --password secret123
./shortlinker update github https://new-github.com --expire 7d --password newpass
```

## 过期时间格式

```bash
1h      # 1小时
1d      # 1天
1w      # 1周
1M      # 1个月
1y      # 1年
1d2h30m # 组合格式
2024-12-31T23:59:59Z  # RFC3339 格式
```

## JSON 格式

```json
[
  {
    "code": "github",
    "target": "https://github.com",
    "created_at": "2024-12-15T14:30:22Z",
    "expires_at": null,
    "click": 0
  }
]
```

## 环境变量

```bash
DATABASE_URL=sqlite://links.db  # 数据库连接 URL
```

> 完整的环境变量配置请参考 [环境变量配置](/config/)

## 批量脚本

```bash
# 备份脚本
./shortlinker export "backup_$(date +%Y%m%d).json"

# 批量导入
while IFS=',' read -r code url; do
    ./shortlinker add "$code" "$url"
done < links.csv
```
