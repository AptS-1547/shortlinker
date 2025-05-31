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
- `--expire <时间>`: 设置过期时间（RFC3339 格式）

### 示例

```bash
# 基本用法
./shortlinker add google https://www.google.com

# 随机短码
./shortlinker add https://www.example.com

# 设置过期时间
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

## 时间格式

### RFC3339 格式

过期时间必须使用 RFC3339 格式：

```bash
# 完整格式
2024-12-31T23:59:59Z

# 带时区
2024-12-31T23:59:59+08:00
```

### 常用时间示例

```bash
# 一天后过期
./shortlinker add daily https://example.com --expire 2024-01-02T00:00:00Z

# 一周后过期
./shortlinker add weekly https://example.com --expire 2024-01-08T00:00:00Z

# 一年后过期
./shortlinker add yearly https://example.com --expire 2025-01-01T00:00:00Z
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
if ./shortlinker add test https://example.com; then
    echo "添加成功"
else
    echo "添加失败"
    exit 1
fi
```
