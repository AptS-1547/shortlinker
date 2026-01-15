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
- `--password <密码>`: 设置密码保护（实验性功能）

**示例**:
```bash
./shortlinker add google https://www.google.com
./shortlinker add https://www.example.com  # 随机短码
./shortlinker add daily https://example.com --expire 1d
./shortlinker add google https://www.google.com --force
./shortlinker add secret https://example.com --password mypass  # 密码保护
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

### generate-config - 生成配置文件

```bash
./shortlinker generate-config [输出路径]
```

生成默认配置文件模板，包含所有可配置选项。

**示例**:
```bash
./shortlinker generate-config           # 生成 config.toml
./shortlinker generate-config myconfig.toml  # 指定文件名
```

### reset-password - 重置管理员密码

```bash
./shortlinker reset-password <新密码>
```

重置管理员 API 密码。新密码会使用 Argon2id 算法哈希后存储到数据库。

**要求**：密码长度至少 8 个字符。

**示例**:
```bash
./shortlinker reset-password "my_new_secure_password"
```

### config - 运行时配置管理（数据库）

`config` 子命令用于直接管理数据库中的运行时配置（与 Web 管理面板使用同一套配置系统）。

> 提示：`config` 命令会把值写入数据库。若要让**正在运行**的服务重新从数据库加载配置，可调用 Admin API `POST /admin/v1/config/reload`，或重启服务。  
> 另外，标记为“需要重启”的配置（如路由前缀、Cookie 配置）即使 reload 也可能无法完全生效，仍建议重启。

常用子命令：

```bash
# 列出所有配置（可选 --category 过滤分类：auth/cookie/features/routes/cors/tracking）
./shortlinker config list
./shortlinker config list --category routes

# 获取单个配置（--json 输出结构化信息）
./shortlinker config get features.random_code_length
./shortlinker config get api.cookie_same_site --json

# 设置/重置配置
./shortlinker config set features.random_code_length 8
./shortlinker config reset features.random_code_length

# 导出/导入配置（JSON）
./shortlinker config export config-backup.json
./shortlinker config import config-backup.json
./shortlinker config import config-backup.json --force   # 跳过交互确认
```

> 安全提醒：配置导出文件会包含敏感字段（如 `api.admin_token`、`api.jwt_secret`、`api.health_token`）的真实值，请妥善保管。

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
- `Enter` 或 `v`：查看详情
- `/`：搜索
- `?`（或 `h`）：帮助
- `x`：导出/导入
- `q`：退出（`Esc` 常用于返回/取消/清除搜索）

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
    "password": null,
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
