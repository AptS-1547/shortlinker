# CLI 命令参考

详细的命令行工具使用说明和参数选项。

## add - 添加短链接

添加新的短链接，支持自定义短码或随机生成。

### 语法

```bash
# 自定义短码
./shortlinker add <短码> <目标URL> [选项]

# 随机短码
./shortlinker add <目标URL> [选项]
```

### 参数

- `<短码>` (可选): 自定义短链接代码
- `<目标URL>` (必需): 目标 URL 地址

### 选项

- `--force`: 强制覆盖已存在的短码
- `--expire <时间>`: 设置过期时间（支持多种格式）

### 示例

```bash
# 基本用法
./shortlinker add google https://www.google.com

# 随机短码
./shortlinker add https://www.example.com

# 使用相对时间格式设置过期时间（推荐）
./shortlinker add daily https://example.com --expire 1d
./shortlinker add weekly https://example.com --expire 1w
./shortlinker add monthly https://example.com --expire 1M
./shortlinker add yearly https://example.com --expire 1y

# 组合时间格式
./shortlinker add complex https://example.com --expire 1d2h30m
./shortlinker add sale https://shop.com --expire 2w3d

# 使用 RFC3339 格式（传统方式）
./shortlinker add temp https://example.com --expire 2024-12-31T23:59:59Z

# 强制覆盖
./shortlinker add google https://www.google.com --force
```

## remove - 删除短链接

删除指定的短链接。

### 语法

```bash
./shortlinker remove <短码>
```

### 示例

```bash
# 删除短链接
./shortlinker remove google
```

## list - 列出短链接

显示所有已创建的短链接。

### 语法

```bash
./shortlinker list
```

### 输出格式

```bash
短链接列表:

  google -> https://www.google.com
  github -> https://github.com
  temp -> https://example.com (过期: 2024-12-31 23:59:59 UTC)

ℹ 共 3 个短链接
```

## update - 更新短链接

更新现有短链接的目标URL和过期时间。

### 语法

```bash
./shortlinker update <短码> <新目标URL> [选项]
```

### 选项

- `--expire <时间>`: 更新过期时间（支持多种格式）

### 示例

```bash
# 更新目标URL
./shortlinker update github https://new-github.com

# 更新URL和过期时间（相对时间格式）
./shortlinker update github https://new-github.com --expire 30d

# 使用组合时间格式
./shortlinker update temp https://example.com --expire 1w2d12h
```

## 时间格式

### 相对时间格式（推荐）

支持简洁的相对时间格式，从当前时间开始计算：

#### 单个时间单位
```bash
1s   # 1秒后过期
5m   # 5分钟后过期
2h   # 2小时后过期
1d   # 1天后过期
1w   # 1周后过期
1M   # 1个月后过期（按30天计算）
1y   # 1年后过期（按365天计算）
```

#### 组合时间格式
```bash
1d2h30m     # 1天2小时30分钟后过期
2w3d        # 2周3天后过期
1y30d       # 1年30天后过期
1h30m15s    # 1小时30分15秒后过期
```

#### 支持的时间单位
| 单位 | 完整形式 | 说明 |
|------|----------|------|
| `s` | `sec`, `second`, `seconds` | 秒 |
| `m` | `min`, `minute`, `minutes` | 分钟 |
| `h` | `hour`, `hours` | 小时 |
| `d` | `day`, `days` | 天 |
| `w` | `week`, `weeks` | 周 |
| `M` | `month`, `months` | 月（30天） |
| `y` | `year`, `years` | 年（365天） |

### RFC3339 格式（兼容）

仍然支持传统的 RFC3339 格式：

```bash
# 完整格式
2024-12-31T23:59:59Z

# 带时区
2024-12-31T23:59:59+08:00
```

### 常用时间示例

```bash
# 短期链接
./shortlinker add flash https://example.com --expire 1h      # 1小时
./shortlinker add daily https://example.com --expire 1d     # 1天

# 中期链接  
./shortlinker add weekly https://example.com --expire 1w    # 1周
./shortlinker add monthly https://example.com --expire 1M   # 1个月

# 长期链接
./shortlinker add yearly https://example.com --expire 1y    # 1年

# 精确时间
./shortlinker add meeting https://zoom.us/j/123 --expire 2h30m  # 2小时30分钟
./shortlinker add sale https://shop.com --expire 2w3d          # 2周3天
```

## 错误代码

| 错误代码 | 说明 |
|----------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 参数错误 |
| 3 | 文件操作错误 |
| 4 | 短码冲突 |
| 5 | 短码不存在 |

## 环境变量

CLI 工具会读取以下环境变量：

```bash
# 随机短码长度
RANDOM_CODE_LENGTH=6

# 存储配置
STORAGE_BACKEND=sqlite
DB_FILE_NAME=links.db

# 日志级别
RUST_LOG=info
```

## 输出控制

### 颜色输出

```bash
# 禁用颜色输出
NO_COLOR=1 ./shortlinker list

# 强制启用颜色
FORCE_COLOR=1 ./shortlinker list
```

### 脚本友好模式

```bash
#!/bin/bash
# 检查命令是否成功
if ./shortlinker add test https://example.com --expire 1d; then
    echo "添加成功"
else
    echo "添加失败"
    exit 1
fi
```
