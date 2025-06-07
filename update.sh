set -e  # 遇到错误时退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_message() {
    local color=$1
    local message=$2
    echo "${color}${message}${NC}"
}

# 显示使用说明
show_usage() {
    echo "使用方法: $0 <version>"
    echo "示例: $0 v0.1.6"
    echo "      $0 0.1.6"
    echo "      $0 v0.1.7-alpha"
    echo "      $0 0.2.0-beta.1"
    echo "      $0 1.0.0-rc.2"
    exit 1
}

# 检查参数
if [ $# -eq 0 ]; then
    print_message $RED "错误: 请提供版本号"
    show_usage
fi

VERSION=$1

# 移除版本号前的 'v' 前缀（如果存在）
VERSION_NUMBER=${VERSION#v}

# 验证版本号格式 (x.y.z 或 x.y.z-prerelease)
if ! [[ $VERSION_NUMBER =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+(\.[0-9]+)*)?$ ]]; then
    print_message $RED "错误: 版本号格式无效。"
    echo "支持的格式:"
    echo "  - 标准版本: x.y.z (例如: 1.0.0)"
    echo "  - 预发布版本: x.y.z-prerelease (例如: 1.0.0-alpha, 1.0.0-beta.1, 1.0.0-rc.2)"
    exit 1
fi

print_message $BLUE "开始更新到版本 $VERSION_NUMBER..."

# 检查是否在git仓库中
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    print_message $RED "错误: 当前目录不是git仓库"
    exit 1
fi

# 检查工作目录是否干净
if [ -n "$(git status --porcelain)" ]; then
    print_message $YELLOW "警告: 工作目录有未提交的更改"
    echo "当前状态:"
    git status --short
    echo
    read -p "是否继续? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_message $RED "操作已取消"
        exit 1
    fi
fi

# 检查Cargo.toml是否存在
if [ ! -f "Cargo.toml" ]; then
    print_message $RED "错误: 找不到 Cargo.toml 文件"
    exit 1
fi

# 获取当前版本
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
print_message $BLUE "当前版本: $CURRENT_VERSION"
print_message $BLUE "目标版本: $VERSION_NUMBER"

# 检查版本是否相同
if [ "$CURRENT_VERSION" = "$VERSION_NUMBER" ]; then
    print_message $YELLOW "警告: 版本号没有变化"
    read -p "是否继续? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_message $RED "操作已取消"
        exit 1
    fi
fi

# 更新 Cargo.toml 中的版本号
print_message $BLUE "更新 Cargo.toml 中的版本号..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \".*\"/version = \"$VERSION_NUMBER\"/" Cargo.toml
else
    # Linux
    sed -i "s/^version = \".*\"/version = \"$VERSION_NUMBER\"/" Cargo.toml
fi

# 验证更新是否成功
NEW_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if [ "$NEW_VERSION" != "$VERSION_NUMBER" ]; then
    print_message $RED "错误: 版本号更新失败"
    exit 1
fi

print_message $GREEN "✓ 版本号已更新为 $VERSION_NUMBER"

# 检查是否有 Cargo.lock，如果有则更新
if [ -f "Cargo.lock" ]; then
    print_message $BLUE "更新 Cargo.lock..."
    cargo check --quiet
    print_message $GREEN "✓ Cargo.lock 已更新"
fi

# Git 操作
print_message $BLUE "添加文件到 git..."
git add Cargo.toml
if [ -f "Cargo.lock" ]; then
    git add Cargo.lock
fi

# 创建提交
COMMIT_MESSAGE="chore: bump version to $VERSION_NUMBER"
print_message $BLUE "创建提交: $COMMIT_MESSAGE"
git commit -m "$COMMIT_MESSAGE"

# 创建标签
TAG_NAME="v$VERSION_NUMBER"
print_message $BLUE "创建标签: $TAG_NAME"
git tag -a "$TAG_NAME" -m "Release $TAG_NAME"

# 推送到远程仓库
print_message $BLUE "推送到远程仓库..."
git push origin $(git branch --show-current)
git push origin "$TAG_NAME"

print_message $GREEN "✅ 更新完成!"
print_message $GREEN "版本 $VERSION_NUMBER 已成功发布"
print_message $BLUE "提交哈希: $(git rev-parse HEAD)"
print_message $BLUE "标签: $TAG_NAME"