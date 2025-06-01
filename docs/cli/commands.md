# CLI 命令参考

详细的命令行工具使用说明和参数选项。

## 基本命令

### add - 添加短链接

```bash
# 自定义短码
./shortlinker add <短码> <目标URL> [选项]

# 随机短码
./shortlinker add <目标URL> [选项]
```

**选项**:
- `--force`: 强制覆盖已存在的短码
- `--expire <时间>`: 设置过期时间

**示例**:
```bash
# 基本用法
./shortlinker add google https://www.google.com

# 随机短码
./shortlinker add https://www.example.com

# 设置过期时间
./shortlinker add daily https://example.com --expire 1d
./shortlinker add sale https://shop.com --expire 2w3d

# 强制覆盖
./shortlinker add google https://www.google.com --force
```

### remove - 删除短链接

```bash
./shortlinker remove <短码>
```

### list - 列出短链接

```bash
./shortlinker list
```

**输出格式**:
```bash
短链接列表:

  google -> https://www.google.com
  github -> https://github.com
  temp -> https://example.com (过期: 2024-12-31 23:59:59 UTC)

ℹ 共 3 个短链接
```

### update - 更新短链接

```bash
./shortlinker update <短码> <新目标URL> [选项]
```

**示例**:
```bash
# 更新目标URL
./shortlinker update github https://new-github.com

# 更新URL和过期时间
./shortlinker update github https://new-github.com --expire 30d
```

## 过期时间格式

### 简单格式（推荐）

```bash
1h    # 1小时
1d    # 1天
1w    # 1周
1M    # 1个月
1y    # 1年
```

### 组合格式

```bash
1d2h30m     # 1天2小时30分钟
2w3d        # 2周3天
1h30m15s    # 1小时30分15秒
```

### RFC3339 格式（兼容）

```bash
2024-12-31T23:59:59Z
2024-12-31T23:59:59+08:00
```

> 💡 **提示**: 更多高级时间格式选项和详细说明，请查看项目文档的"高级用法"部分

## 常用时间示例

```bash
# 短期链接
./shortlinker add flash https://example.com --expire 1h      # 1小时
./shortlinker add daily https://example.com --expire 1d     # 1天

# 中长期链接  
./shortlinker add weekly https://example.com --expire 1w    # 1周
./shortlinker add monthly https://example.com --expire 1M   # 1个月

# 精确时间
./shortlinker add meeting https://zoom.us/j/123 --expire 2h30m
./shortlinker add sale https://shop.com --expire 2w3d
```

## 热重载机制

CLI 操作会自动通知服务器重载配置：

```bash
# Unix/Linux 系统 - 自动发送 SIGUSR1 信号
./shortlinker add new https://example.com
# 输出：✓ 已添加短链接: new -> https://example.com
#      ℹ 已通知服务器重新加载配置

# Windows 系统 - 自动创建触发文件
./shortlinker add new https://example.com
```

## 错误代码

| 错误代码 | 说明 |
|----------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 参数错误 |
| 4 | 短码冲突 |
| 5 | 短码不存在 |

## 环境变量

CLI 工具读取的主要环境变量：

```bash
RANDOM_CODE_LENGTH=6      # 随机短码长度
STORAGE_BACKEND=sqlite    # 存储后端类型
DB_FILE_NAME=links.db     # 数据库文件路径
RUST_LOG=info            # 日志级别
```

> 完整的环境变量配置请参考 [环境变量配置](/config/)

## 脚本集成

### 批量操作
```bash
#!/bin/bash
# 批量导入链接
while IFS=',' read -r code url; do
    ./shortlinker add "$code" "$url"
done < links.csv
```

### 错误检查
```bash
if ./shortlinker add test https://example.com --expire 1d; then
    echo "添加成功"
else
    echo "添加失败"
    exit 1
fi
```

## 进程管理

### 检查服务状态
```bash
# Unix 系统
if [ -f shortlinker.pid ]; then
    echo "服务器 PID: $(cat shortlinker.pid)"
else
    echo "服务器未运行"
fi
```

### 容器环境
在 Docker 容器中，进程管理会自动处理容器重启，无需手动处理。
