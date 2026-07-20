# 贡献指南

感谢您对 Shortlinker 项目的关注！我们欢迎任何形式的贡献，包括但不限于：

- 🐛 报告 bug
- 💡 提出新功能建议
- 📖 改进文档
- 🔧 提交代码修复或功能实现
- 🌐 翻译文档
- ⭐ 给项目点星

## 📋 目录

- [开发环境设置](#开发环境设置)
- [项目结构](#项目结构)
- [开发流程](#开发流程)
- [代码规范](#代码规范)
- [提交规范](#提交规范)
- [测试](#测试)
- [文档](#文档)
- [发布流程](#发布流程)
- [问题报告](#问题报告)
- [功能请求](#功能请求)

## 开发环境设置

### 前置要求

- **Rust**: >= 1.82.0
- **Bun**: 1.3.14（管理面板开发）
- **Git**: 最新版本
- **Docker**: (可选) 用于容器化测试

### 安装 Rust

```bash
# 安装 Rust 和 Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 克隆项目

```bash
git clone https://github.com/AptS-1547/shortlinker.git
cd shortlinker
```

### 安装开发工具

```bash
# 代码格式化工具
rustup component add rustfmt

# 代码检查工具
rustup component add clippy

# 交叉编译工具 (可选)
cargo install cross
```

### 第一次运行

```bash
# 安装并构建管理面板
cd admin-panel
bun install --frozen-lockfile --registry https://registry.npmjs.org
bun run build
cd ..

# 编译项目
cargo build

# 运行项目
cargo run

# 运行测试
cargo test
```

## 项目结构

```
shortlinker/
├── src/
│   ├── main.rs              # 程序入口点
│   ├── server.rs            # HTTP 服务器实现
│   ├── cli.rs               # 命令行接口
│   ├── config.rs            # 配置管理
│   ├── storage.rs           # 数据存储
│   ├── admin.rs             # Admin API
│   └── utils.rs             # 工具函数
├── admin-panel/             # React 管理面板
│   ├── src/                 # 前端源码
│   └── package.json         # 前端依赖与脚本
├── docs/                    # 文档源码 (VitePress)
├── .github/
│   └── workflows/           # GitHub Actions 工作流
├── Dockerfile               # Docker 镜像构建
├── Cargo.toml              # Rust 项目配置
├── README.md               # 项目说明
├── README.zh.md            # 中文说明
└── CONTRIBUTING.md         # 贡献指南
```

## 开发流程

### 1. Fork 和 Clone

```bash
# 1. 在 GitHub 上 Fork 项目
# 2. Clone 你的 Fork
git clone https://github.com/YOUR_USERNAME/shortlinker.git
cd shortlinker

# 3. 添加上游仓库
git remote add upstream https://github.com/AptS-1547/shortlinker.git
```

### 2. 创建功能分支

```bash
# 从 master 分支创建新分支
git checkout master
git pull upstream master
git checkout -b feature/your-feature-name
```

### 3. 开发和测试

```bash
# 开发过程中频繁测试
cargo test
cargo clippy
cargo fmt

# 运行服务器测试
cargo run
```

### 4. 提交更改

```bash
# 添加更改
git add .

# 提交 (遵循提交规范)
git commit -m "feat: add new feature description"

# 推送到你的 Fork
git push origin feature/your-feature-name
```

### 5. 创建 Pull Request

1. 在 GitHub 上打开你的 Fork
2. 点击 "Compare & pull request"
3. 填写 PR 模板
4. 等待代码审查

## 代码规范

### Rust 代码风格

我们使用标准的 Rust 代码风格：

```bash
# 自动格式化代码
cargo fmt

# 检查代码质量
cargo clippy

# 检查是否符合格式要求
cargo fmt -- --check

# 运行所有检查
cargo clippy -- -D warnings
```

### 代码组织原则

1. **模块化**: 每个功能模块单独文件
2. **错误处理**: 使用 `Result<T, E>` 类型
3. **文档注释**: 为公共 API 添加文档
4. **测试**: 为新功能添加单元测试

### 示例代码风格

```rust
/// 添加新的短链接
/// 
/// # Arguments
/// 
/// * `code` - 短链接代码
/// * `target` - 目标 URL
/// * `expires_at` - 可选的过期时间
/// 
/// # Returns
/// 
/// 返回操作结果
pub fn add_link(
    code: &str, 
    target: &str, 
    expires_at: Option<DateTime<Utc>>
) -> Result<(), StorageError> {
    // 实现逻辑...
    Ok(())
}
```

## 提交规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

### 提交格式

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### 提交类型

- `feat`: 新功能
- `fix`: bug 修复
- `docs`: 文档更新
- `style`: 代码格式修改
- `refactor`: 代码重构
- `test`: 测试相关
- `chore`: 构建过程或辅助工具的变动

### 示例

```bash
# 新功能
git commit -m "feat(cli): add batch import command"

# Bug 修复
git commit -m "fix(server): handle empty short code correctly"

# 文档更新
git commit -m "docs: update API documentation"

# 重构
git commit -m "refactor(storage): simplify JSON handling"
```

## 测试

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 运行测试并显示输出
cargo test -- --nocapture

# 生成测试覆盖率报告 (需要 tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### 编写测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_link() {
        // 测试添加链接功能
        let result = add_link("test", "https://example.com", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_url() {
        // 测试无效 URL 处理
        let result = add_link("test", "invalid-url", None);
        assert!(result.is_err());
    }
}
```

### 集成测试

```bash
# 运行集成测试
cargo test --test integration_tests

# 手动测试服务器
cargo run &
curl -I http://localhost:8080/
kill %1
```

## 文档

### 代码文档

```bash
# 生成文档
cargo doc --open

# 检查文档链接
cargo doc --no-deps
```

### 用户文档

用户文档使用 VitePress 构建，位于 `docs/` 目录：

```bash
# 安装 Node.js 依赖
cd docs
npm install

# 本地开发
npm run dev

# 构建文档
npm run build
```

### 文档编写规范

1. 使用清晰的标题层次
2. 提供完整的代码示例
3. 包含常见问题解答
4. 保持中英文文档同步

## 发布流程

### 版本号规范

我们使用 [语义化版本](https://semver.org/)：

- `MAJOR.MINOR.PATCH`
- `1.0.0`: 主要版本，不兼容的 API 修改
- `0.1.0`: 次要版本，向下兼容的功能性新增
- `0.0.1`: 修订版本，向下兼容的问题修正

### 发布步骤

1. **更新版本号**：修改 `Cargo.toml`
2. **更新 CHANGELOG**：记录本版本的变化
3. **创建 Git 标签**：`git tag v1.0.0`
4. **推送标签**：`git push origin v1.0.0`
5. **GitHub Actions** 自动构建和发布

### 预发布测试

```bash
# 本地构建测试
cargo build --release

# Docker 构建测试
docker build -t shortlinker-test .

# 交叉编译测试
cross build --release --target x86_64-unknown-linux-musl
```

## 问题报告

### Bug 报告

当报告 bug 时，请包含以下信息：

1. **环境信息**：
   - 操作系统和版本
   - Rust 版本
   - Shortlinker 版本

2. **重现步骤**：
   - 详细的操作步骤
   - 输入数据示例
   - 预期行为 vs 实际行为

3. **错误信息**：
   - 完整的错误消息
   - 相关日志输出
   - 堆栈跟踪（如果有）

### Bug 报告模板

```markdown
## Bug 描述
简短描述遇到的问题

## 环境信息
- OS: [e.g. Ubuntu 22.04]
- Rust: [e.g. 1.82.0]
- Shortlinker: [e.g. v0.0.5]

## 重现步骤
1. 步骤一
2. 步骤二
3. 步骤三

## 预期行为
描述你期望发生的情况

## 实际行为
描述实际发生的情况

## 错误信息
```
粘贴错误信息或日志
```

## 额外信息
添加任何其他有用的信息
```

## 功能请求

### 功能请求模板

```markdown
## 功能描述
清晰描述你希望添加的功能

## 使用场景
描述这个功能解决什么问题

## 实现建议
如果有想法，描述可能的实现方式

## 替代方案
是否考虑过其他解决方案

## 额外信息
添加任何其他相关信息
```

## 社区准则

### 行为规范

1. **尊重他人**：友善对待所有贡献者
2. **建设性反馈**：提供有用的建议和批评
3. **包容性**：欢迎不同背景的贡献者
4. **专业性**：保持专业的交流态度

### 交流渠道

- **GitHub Issues**: 报告 bug 和功能请求
- **GitHub Discussions**: 社区讨论
- **Pull Requests**: 代码审查和讨论

## 获取帮助

如果在贡献过程中遇到问题，可以：

1. 查看现有的 [Issues](https://github.com/AptS-1547/shortlinker/issues)
2. 创建新的 [Discussion](https://github.com/AptS-1547/shortlinker/discussions)
3. 联系项目维护者

## 致谢

感谢所有为 Shortlinker 项目做出贡献的开发者！

---

再次感谢您的贡献！每一个贡献都让 Shortlinker 变得更好。🎉
