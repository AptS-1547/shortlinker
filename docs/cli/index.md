# CLI 命令行工具

Shortlinker 提供了直观易用的命令行工具，用于管理短链接。

## 工具特性

- 🎨 **彩色输出** - 清晰的视觉反馈
- 🔄 **实时同步** - 命令执行立即生效  
- ⚡ **快速响应** - 本地文件操作，毫秒级响应
- 🛡️ **错误处理** - 详细的错误信息和建议

## 基本语法

```bash
./shortlinker <command> [arguments] [options]
```

## 命令概览

| 命令 | 功能 | 示例 |
|------|------|------|
| `add` | 添加短链接 | `./shortlinker add github https://github.com` |
| `remove` | 删除短链接 | `./shortlinker remove github` |
| `list` | 列出所有链接 | `./shortlinker list` |

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

### 高级功能
```bash
# 随机短码
./shortlinker add https://example.com
# 输出：✓ 已添加短链接: aB3dF1 -> https://example.com

# 设置过期时间
./shortlinker add sale https://shop.com/sale --expire 2024-12-25T00:00:00Z

# 强制覆盖
./shortlinker add docs https://new-docs.com --force
```

## 输出说明

### 成功状态
- ✅ 绿色文本表示操作成功
- 🔵 蓝色文本显示信息提示

### 错误状态  
- ❌ 红色文本显示错误信息
- 💡 提供解决建议

### 示例输出
```bash
$ ./shortlinker add github https://github.com
✓ 已添加短链接: github -> https://github.com

$ ./shortlinker add github https://gitlab.com
❌ 错误: 短码 'github' 已存在，当前指向: https://github.com
💡 如需覆盖，请使用 --force 参数
```

## 环境变量支持

CLI 工具读取与服务器相同的环境变量：

```bash
# 自定义存储路径
LINKS_FILE=./custom-links.json ./shortlinker list

# 自定义随机码长度
RANDOM_CODE_LENGTH=8 ./shortlinker add https://example.com
```

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
if ./shortlinker add test https://example.com; then
    echo "添加成功"
else
    echo "添加失败"
    exit 1
fi
```

## 下一步

- 📖 查看 [详细命令参考](/cli/commands) 了解所有选项
- ⚙️ 学习 [配置说明](/config/) 自定义行为
