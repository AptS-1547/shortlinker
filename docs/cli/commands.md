# CLI 命令参考

详细的命令行工具使用说明和参数选项。

## 常见任务导航

- **第一次上手**：`add` → `list` → `update` → `remove`
- **批量迁移**：`import` / `export`
- **运维管理**：`config` / `reset-password`
- **交互管理**：`tui`

> 如果你只想快速可视化管理，建议直接使用 [TUI 界面](/cli/tui)。

## 核心命令（推荐阅读顺序）

### add - 添加短链接

```bash
./shortlinker add <短码> <目标URL> [选项]
./shortlinker add <目标URL> [选项]  # 随机短码
```

> 说明：短码需满足格式约束（长度 ≤ 128，字符集 `[a-zA-Z0-9_.-/]`），且不能与保留路由前缀冲突（默认 `admin`/`health`/`panel`，由 `routes.*_prefix` 决定）。

**选项**：
- `--force`：强制覆盖已存在的短码
- `--expire <时间>`：设置过期时间
- `--password <密码>`：设置密码保护（实验性功能）

**示例**：
```bash
./shortlinker add google https://www.google.com
./shortlinker add https://www.example.com  # 随机短码
./shortlinker add daily https://example.com --expire 1d
./shortlinker add google https://www.google.com --force
./shortlinker add secret https://example.com --password mypass
```

### list - 列出短链接

```bash
./shortlinker list
```

### update - 更新短链接

```bash
./shortlinker update <短码> <新目标URL> [选项]
```

**选项**：
- `--expire <时间>`：设置新的过期时间
- `--password <密码>`：设置或更新密码

**示例**：
```bash
./shortlinker update github https://new-github.com
./shortlinker update github https://new-github.com --expire 30d
./shortlinker update github https://new-github.com --password secret123
```

### remove - 删除短链接

```bash
./shortlinker remove <短码>
```

### import - 导入短链接

```bash
./shortlinker import <文件路径> [选项]
```

**选项**：
- `--force`：强制覆盖已存在的短码

**示例**：
```bash
./shortlinker import backup.csv
./shortlinker import backup.csv --force
```

> 默认使用 CSV 格式；`.json` 仅为兼容旧格式（将于 v0.5.0 移除）。

### export - 导出短链接

```bash
./shortlinker export [文件路径]
```

**示例**：
```bash
./shortlinker export
./shortlinker export backup.csv
```

### help - 查看帮助

```bash
./shortlinker help
```

## 运维命令

### config - 配置管理

`config` 子命令用于管理 Shortlinker 配置。

#### config generate - 生成配置文件

```bash
./shortlinker config generate [输出路径] [选项]
```

生成**启动配置**（`config.toml`）模板，包含 `server` / `database` / `cache` / `logging` / `analytics` 等配置项。
运行时配置（如 `features.*`、`api.*`、`routes.*`、`cors.*`）存储在数据库中，不在该文件内。

> 注意：此命令不需要数据库连接，可以在首次部署时直接使用。

**选项**：
- `--force`：跳过确认，强制覆盖已存在的文件

**示例**：
```bash
./shortlinker config generate                       # 生成 config.example.toml
./shortlinker config generate config.toml           # 文件存在时会交互确认
./shortlinker config generate config.toml --force   # 强制覆盖
```

#### config list/get/set/reset - 运行时配置管理（数据库）

以下子命令用于直接管理数据库中的运行时配置（与 Web 管理面板使用同一套配置系统）。

> 提示：`config` 命令会把值写入数据库。若要让**正在运行**的服务重新从数据库加载配置，可调用 Admin API `POST /admin/v1/config/reload`，或重启服务。
> 标记为"需要重启"的配置（如 `routes.*`、`click.*`、`cors.*`）即使 reload 也不会热生效，仍需要重启。

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
./shortlinker config import config-backup.json --force
```

> 安全提醒：配置导出文件会包含敏感字段（如 `api.admin_token`、`api.jwt_secret`、`api.health_token`）的真实值，请妥善保管。

### reset-password - 重置管理员密码

```bash
./shortlinker reset-password [选项]
```

重置管理员 API 密码。新密码会使用 Argon2id 算法哈希后存储到数据库。

**要求**：密码长度至少 8 个字符。

**示例**：
```bash
# 交互式输入（推荐）
./shortlinker reset-password

# 从 stdin 读取（脚本）
echo "my_new_secure_password" | ./shortlinker reset-password --stdin

# 通过参数传入（不推荐：会出现在 shell history）
./shortlinker reset-password --password "my_new_secure_password"
```

## 交互界面

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

> 💡 详细使用说明请参考 [TUI 使用指南](/cli/tui)。

## 进阶与自动化

### 过期时间格式

```bash
1h      # 1小时
1d      # 1天
1w      # 1周
1M      # 1个月
1y      # 1年
1d2h30m # 组合格式
2024-12-31T23:59:59Z  # RFC3339 格式
```

### 导入导出格式

**CSV（默认）**

导出文件包含 header，字段：
`code,target,created_at,expires_at,password,click_count`

```csv
code,target,created_at,expires_at,password,click_count
github,https://github.com,2024-12-15T14:30:22Z,,,
```

**JSON（兼容旧格式，已废弃）**

> `.json` 仅为兼容旧格式（将于 v0.5.0 移除）。

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

### 热重载说明

链接数据变更命令（如 `add` / `update` / `remove` / `import`）会尝试通知运行中的服务刷新内存缓存。

> 注意：这不等同于运行时配置热更新。通过 `./shortlinker config set` 改动配置后，请调用 Admin API `POST /admin/v1/config/reload` 或重启服务。

### 数据库配置

CLI 会读取当前工作目录的 `config.toml` 来连接数据库。如需指定数据库连接，请在 `config.toml` 中设置：

```toml
[database]
database_url = "sqlite://links.db"
```

> 更多配置见 [配置指南](/config/)。

### 批量脚本

```bash
# 备份脚本
./shortlinker export "backup_$(date +%Y%m%d).csv"

# 批量导入
while IFS=',' read -r code url; do
    ./shortlinker add "$code" "$url"
done < links.csv
```
