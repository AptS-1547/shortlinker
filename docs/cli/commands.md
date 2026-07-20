# CLI 命令参考

详细的命令行工具使用说明和参数选项。

## 常见任务导航

- **第一次上手**：`add` → `list` → `update` → `remove`
- **批量迁移**：`import` / `export`
- **运维管理**：`config` / `reset-password`
## 全局参数

所有 CLI 子命令都支持以下全局参数：

- `-s, --socket <路径>`：覆盖 IPC socket 路径（Unix）或命名管道路径（Windows）

> 优先级：CLI `--socket` > `config.toml` 的 `ipc.socket_path` > 平台默认值。

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

> 仅支持 CSV 导入；请使用 `.csv` 文件。

### export - 导出短链接

```bash
./shortlinker export [文件路径]
```

**示例**：
```bash
./shortlinker export
./shortlinker export backup.csv
```

> 不指定文件路径时，会生成 `shortlinks_export_YYYYMMDD_HHMMSS.csv`。

### help - 查看帮助

```bash
./shortlinker help
```

### status - 查看服务状态（IPC）

```bash
./shortlinker status
./shortlinker --socket /tmp/custom.sock status
```

当服务可达时，会显示：版本、运行时长、是否正在重载、最近一次数据/配置重载时间、链接总数。
如果 IPC 不可达（服务未启动、`ipc.enabled=false`、路径不一致等），会提示“Server is not running”。

## 运维命令

### config - 配置管理

`config` 子命令用于管理 Shortlinker 配置。

#### config generate - 生成配置文件

```bash
./shortlinker config generate [输出路径] [选项]
```

生成**启动配置**（`config.toml`）模板，包含 `server` / `database` / `cache` / `logging` / `analytics` / `ipc` 等配置项。
运行时配置（如 `features.*`、`api.*`、`routes.*`、`click.*`、`cors.*`、`analytics.*`、`utm.*`、`cache.*`）存储在数据库中，不在该文件内。

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

> 提示：`config set/reset` 仅在“无需重启”的键写库后**自动尝试**通过 IPC 触发 `Config` 重载。
> `config import` 会在导入后统一进行一次 `Config` 重载尝试（best-effort）。
> 若 IPC 不可达（服务未运行、`ipc.enabled=false`、socket 路径不一致等），请手动调用 Admin API `POST /admin/v1/config/reload`。
> 标记为“需要重启”的配置（如 `routes.*`、`click.*`、`cors.*`、`cache.*`）即使 reload 也不会热生效，仍需要重启。

常用子命令：

```bash
# 列出配置（纯文本输出按 auth/cookie/features/routes/cors/tracking 分组）
./shortlinker config list
./shortlinker config list --category routes
# 如需完整键集合（含 analytics/utm/cache），使用 --json
./shortlinker config list --json

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

### 导入/导出格式（links）

**CSV（默认）**

导出文件包含 header，字段：
`code,target,created_at,expires_at,password,click_count`

```csv
code,target,created_at,expires_at,password,click_count
github,https://github.com,2024-12-15T14:30:22Z,,,
```

### 热重载说明

当服务正在运行且 IPC 可达时，链接管理命令会优先通过 IPC 在服务进程内执行，避免“DB 已写入但服务缓存未更新”的窗口。

若 IPC 不可达，CLI 会回退为本地数据库操作（适合离线维护）；此时如果线上服务仍在运行，需要你手动让服务刷新数据（通常重启服务）。

> 注意：运行时配置改动与链接数据改动是两条路径。`config set/reset` 仅对“无需重启”的键尝试 `Config` 重载；`config import` 导入后会统一尝试一次 `Config` 重载；“需要重启”的键仍必须重启。

### 数据库配置

CLI 会读取当前工作目录的 `config.toml` 来连接数据库。如需指定数据库连接，请在 `config.toml` 中设置：

```toml
[database]
database_url = "sqlite://shortlinks.db"
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
