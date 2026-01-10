# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [v0.3.0-alpha.3] - 2026-01-11

### Added
- **动态配置系统** - 实现基于数据库的运行时配置管理，支持配置热重载
  - 新增 `system_config` 和 `config_history` 数据表存储配置及变更历史
  - 添加配置管理 API 端点（`GET/PUT /api/admin/config`），支持通过 API 或管理面板动态修改配置
  - 支持配置热更新，无需重启服务即可应用大部分配置变更（JWT 密钥、API 令牌、路由前缀等）
  - 实现配置版本控制和变更历史追踪
  - 首次启动时自动从 `config.toml` 迁移配置到数据库
- 引入 `arc-swap` 依赖，实现配置的原子更新和线程安全访问

### Changed
- **配置架构重构** - 明确区分启动配置（需重启）和动态配置（可热更新）
  - 启动配置：数据库连接、服务器绑定地址、日志级别等
  - 动态配置：JWT 设置、API 令牌、CORS、路由前缀等功能配置
- 重构认证中间件和健康检查中间件，从运行时配置读取设置，支持动态更新
- 更新配置文档（`docs/config/index.md`），添加详细的配置说明和迁移指南
- 简化 `config.example.toml`，移除已迁移到数据库的动态配置项

### Improved
- 将错误测试从中文翻译为英文，提升代码国际化水平
- 修复配置文档中的代码示例，添加缺失的 `use` 语句

### Migration Notes
**⚠️ 重要：首次启动 v0.3.0-alpha.3 时，系统会自动从 `config.toml` 迁移配置到数据库**

1. 升级后首次启动时，系统会自动执行配置迁移
2. 迁移完成后，数据库配置将作为配置源，`config.toml` 中的动态配置项不再生效
3. 后续可通过管理面板或 API 修改配置，变更会实时生效且持久化到数据库
4. 如需重置为 `config.toml` 配置，请删除数据库后重新启动

查看完整配置文档：`docs/config/index.md`

## [v0.3.0-alpha.2] - 2026-01-07

### Changed
- chore(deps): bump preact

## [v0.3.0-alpha.1] - 2026-01-07

### Added
- JWT 认证支持

### Changed
- 升级 shortlinker 版本至 0.3.0-alpha.1
- 升级依赖并添加 JWT 认证支持
- 更新 admin-panel 子模块到新提交

### Refactored
- 统一 API 错误码格式，分离健康检查数据与响应结构
- 添加 JWT 认证系统替换 Bearer Token

## [v0.2.3-alpha.3] - 2025-10-24

### Changed
- Update Dockerfile

### Refactored
- 重构项目模块结构
- 重构 TUI 应用模块结构
- 重构 TUI 界面和构建配置

## [v0.2.3-alpha.2] - 2025-10-24

### Changed
- 更新版本号并完善 Docker 构建
- 更新依赖包并改进 TUI 功能

## [v0.2.3-alpha.1] - 2025-10-24

### Changed
- ORM 重构 (#25)
- chore(deps): bump vite (#24)

## [v0.2.2] - 2025-10-21

### Added
- 增强配置管理和日志系统

### Changed
- 优化构建流程并添加版本信息显示
- 清理已弃用代码并升级依赖
- chore(deps): bump vite (#23)

### Docs
- improve documentation accuracy and completeness

## [v0.2.1] - 2025-10-21

### Added
- 更新开发中功能状态，完成日志记录与轮转
- 增强日志系统和点击统计配置

### Changed
- 更新项目版本和依赖项
- 更新依赖并优化代码结构
- Update vitepress.yml

### Refactored
- 重构系统架构和平台抽象层
- 优化缓存接口参数和简化代码结构
- 优化点击管理器并发控制和性能监控
- 优化日志级别和代码结构

### Docs
- 更新文档以反映存储后端和 TUI 功能变化

## [v0.2.0] - 2025-09-01

### Added
- 添加完整的 TUI 终端用户界面
  - 支持链接的查看、添加、编辑、删除操作
  - 添加导入导出功能，支持 JSON 格式数据交换
  - 包含过期时间管理和密码保护等高级特性
  - 提供丰富的用户交互和状态反馈机制
- 添加 TUI 支持并改进错误处理
- 添加 TOML 配置文件支持
- 添加 Rust 开发相关的 VSCode 扩展

### Changed
- 更新依赖和构建配置
- 实现功能模块化构建配置
- 升级版本至 0.2.0-alpha.2 并优化代码

### Fixed
- 增加 flush 过程中的调试信息以改善日志可读性 (Fixed #14)

### Refactored
- 重构缓存系统命名和初始化逻辑
- 简化缓存配置机制
- 重构配置系统和移除废弃组件
- 重构点击统计后台任务启动方式 (Fixed #14)

## [v0.2.0-alpha.3] - 2025-08-30

### Added
- 添加完整的 TUI 终端用户界面
- 添加 TUI 支持并改进错误处理
- 添加详细的 TUI 使用文档

### Changed
- 实现功能模块化构建配置
- 升级版本至 0.2.0-alpha.2 并优化代码

### Refactored
- 重构缓存系统命名和初始化逻辑

## [v0.2.0-alpha.2] - 2025-08-27

### Added
- 添加 TOML 配置文件支持

### Fixed
- 增加 flush 过程中的调试信息以改善日志可读性 (Fixed #14)

### Refactored
- 简化缓存配置机制
- 重构配置系统和移除废弃组件

## [v0.2.0-alpha.1] - 2025-08-27

### Changed
- 更新依赖和构建配置

## [v0.1.7-alpha.7] - 2025-06-21

### Added
- 点击统计功能 (#17)

### Fixed
- 优化 Redis 客户端创建和缓存获取逻辑

### Reverted
- Revert "Feat click count (#15)" (#16)

## [v0.1.7-alpha.6] - 2025-06-21

### Added
- 添加 uuid 依赖并创建项目路线图文档

### Changed
- Refactor system and redis (#13)
- Update Dockerfile

### Fixed
- 更新依赖项版本并移除不再使用的服务器命令

## [v0.1.7-alpha.5] - 2025-06-19

### Added
- 添加对 MySQL 和 PostgreSQL 存储后端的支持
- 添加子模块支持并更新代码检出步骤
- 添加管理面板界面图片到文档

### Changed
- 更新 Dockerfile 和依赖项，提升构建效率和兼容性

### Fixed
- 移除 Dockerfile 中的 .git 目录并更新 admin-panel 子项目的提交引用
- 移除 admin-panel 中的 .git 目录并更新数据库文件名环境变量
- 更新管理面板子项目的提交引用
- 更新 bug 报告模板中的版本选项以反映最新版本
- 优化开发分支和预发布版本的判断逻辑
- 修改路由匹配规则以支持更灵活的链接代码格式

### Refactored
- migrate from rusqlite to sqlx for SQLite backend
- 删除前端文件夹

## [v0.1.7-alpha.4] - 2025-06-11

### Refactored
- 使用 rust-embed 重构前端静态文件服务

## [v0.1.7-alpha.3] - 2025-06-10

### Added
- 添加语言切换组件，支持日语和语言配置更新

### Changed
- enhance dark mode support in LoginView and update dependencies
- Enhance UI/UX for LinksView and LoginView with modern design elements

### Fixed
- 修复预启动处理时间输出，确保正确计算毫秒数

### Style
- 重命名函数和变量以提高可读性

## [v0.1.7-alpha.2] - 2025-06-08

### Added
- Add unit tests for Rust middleware and utils, plus Vue store and service (#9)
- Add docs for enabling admin panel (#8)

### Changed
- 更新 .gitignore，添加 .yarn/ 目录；删除 install-state.gz 文件

### Fixed
- 更新 OUT_DIR 环境变量错误提示；将 admin 面板构建命令从 npm 改为 yarn

### Refactored
- 优化启动流程，重构缓存和存储初始化逻辑；添加全局更新锁以防止并发更新

### Docs
- 更新基准测试版本号至 v0.1.7-alpha.1
- 更新管理面板和 Cloudflare Worker 文档，移除多余信息

## [v0.1.7-alpha.1] - 2025-06-08

### Added
- Add frontend build process and integrate with Rust backend
- 添加文件和 SQLite 存储插件，重构存储注册机制
- Implement layered caching with L1 and L2 cache plugins
- 添加 L1 和 L2 缓存插件的默认实现，重构缓存清理和加载逻辑
- Implement CORS support and admin authentication in the Actix web application
- update LoginView for improved UI and internationalization

### Changed
- 更新前端构建流程，使用 Node.js 20，改为 npm，增加版本号支持
- Update README.md

### Fixed
- 使用 vec 保证 get_all_links 的排列顺序

### Refactored
- 重构缓存系统架构并引入点击追踪
- 重构缓存加载逻辑，合并 L1 和 L2 缓存加载方法
- 重构存储插件注册机制，使用宏简化插件注册过程
- Middleware 重构

### Docs
- 更新 README.md 中的 CPU 信息和基准测试结果
- 更新 README.md 格式，优化环境和配置变量部分的可读性

## [v0.1.6] - 2025-06-06

### Added
- 添加布隆过滤器支持，优化链接重定向性能
- 在 CLI 模式下跳过布隆过滤器检查
- 在 SQLite 存储中添加布隆过滤器初始化逻辑 (fix #5)

### Changed
- 将存储工厂和存储后端的创建方法改为异步，优化存储初始化流程

### Refactored
- 更新重定向服务，移除布隆过滤器相关代码
- 修改文件存储和 Sled 存储的构造函数为异步
- 统一日志输出格式，提升代码可读性

## [v0.1.5] - 2025-06-05

### Added
- 添加链接查询功能，支持分页和过滤条件
- 添加 admin-panel 目录，包含 Next.js 应用的基础配置和组件
- 添加静态生成参数函数并更新 Next.js 配置
- Implement cookie-based admin login

### Changed
- 优化身份验证和随机代码生成逻辑，使用 OnceLock 缓存环境变量值
- Translate log messages to English

### Fixed
- 更新 cargo.lock 依赖

### Refactored
- 移除未使用的静态生成函数和管理员登录逻辑
- Refactor code structure for improved readability and maintainability

### Style
- 格式化代码，调整导入语句的排版；移除未使用的 debug 导入

### Docs
- document server management commands

## [v0.1.4] - 2025-06-04

### Added
- 添加 CPU_COUNT 环境变量以配置工作线程数量，优化服务器性能
- 添加 colored 库以增强 CLI 输出，移除自定义颜色工具
- 更新 Rust 构建工作流，添加 ARM64 支持，优化交叉编译配置

### Changed
- 更新 .gitignore，添加 shortlinker.pid 和 cargo-flamegraph.trace/，优化文件管理
- 更新 VitePress 工作流，支持手动触发，切换至自托管环境，升级 Node.js 版本
- 更新包描述和作者信息

### Refactored
- 重构多个文件中的 println! 语句，提升代码可读性
- 调整 SQLite 存储配置，优化数据库性能

### Docs
- 更新文档，修正 HTTP 跳转类型为 307，添加数据导入导出命令示例

## [v0.1.3] - 2025-06-03

### Added
- 添加短链接导入导出功能，优化 SQLite 存储配置，大幅度优化编译大小
- sqlite storage 添加 moka TTL 缓存机制
- 添加 APT 包缓存和安装步骤，优化 ARM64 交叉编译器配置
- 为 ARM64 目标添加专用工具链安装，更新 musl 编译器配置
- 更新 Rust 发布工作流以支持 musl 编译，优化交叉编译配置

### Changed
- 更新依赖项，优化文件存储和 SQLite 存储实现，添加日期解析错误处理
- Update ROADMAP.md
- Update rust-release.yml

### Fixed
- 更新 rusqlite 依赖项以启用捆绑特性，添加 cc 依赖
- 修复 UNIX_SOCKET_PATH 环境变量名称，添加 socket 卷到 Dockerfile

## [v0.1.2] - 2025-06-02

### Added
- 添加 Unix socket 支持，优化 HTTP 连接保持策略
- 添加点击量计数器功能到存储模块
- 添加 r2d2 和 r2d2_sqlite 依赖，重构数据库连接管理
- 添加 Docker 部署和相关配置文档

### Changed
- 更新依赖并改进测试和中间件

### Fixed
- UNIX_SOCKET_PATH 在 Windows 下编译的问题
- 更新编辑链接的 GitHub 分支为 master

### Refactored
- 优化短链接存储逻辑

### Docs
- 更新文档以反映新配置选项
- Refactor documentation and improve clarity across multiple files

## [v0.1.1] - 2025-06-01

### Added
- 添加对多种过期时间格式的支持，包括相对时间和 RFC3339 格式

### Changed
- 优化 Rust 编译选项，提升构建性能
- 移除 Dockerfile 中的 links.json 复制步骤

## [v0.1.0] - 2025-06-01

### Added
- 添加 SQLite 存储后端实现，支持短链接的持久化存储
- 添加存储后端名称获取功能
- 添加错误处理机制，重构存储后端以返回自定义错误类型
- Implement CLI for shortlinker with link management and server control

### Changed
- 更新存储配置，支持多种存储后端并修改默认数据库路径
- 更新 Dockerfile 环境变量配置
- 更新短链接服务，重命名处理函数并添加库配置

### Fixed
- 修复 Admin API 可用性日志格式
- 更新 GitHub Actions 配置以获取完整的 Git 历史和所有标签
- 添加交叉编译工具和配置以支持多种目标平台

### Refactored
- 重构存储模块并修复 dotenv 加载问题
- 重构存储后端，支持依赖注入并更新相关代码
- 重构 CLI 相关模块，优化帮助信息和链接管理，添加锁文件管理功能
- 重构服务模块，添加管理和重定向服务
- 公开解析器方法并重构测试结构
- 删除测试文件和基准测试，清理代码库

### Style
- 优化代码结构和格式，调整导入顺序，清理多余空行，增强可读性

### Docs
- 统一存储后端配置文档
- 统一存储配置并添加全面测试覆盖
- 添加 Cloudflare Worker 的英文和中文文档
- 添加文档以支持短链接管理面板的使用说明
- 添加英文文档支持并完善文档内容
- 添加多存储后端支持文档

## [v0.0.6] - 2025-05-30

### Added
- 更新短链接存储实现，添加示例链接返回功能
- 添加讨论、创意、问答和展示模板，增强社区互动和反馈机制
- 添加贡献指南文档，提供项目贡献流程和规范
- 优化 changelog 生成逻辑，支持根据提交类型分类并添加比较链接
- 优化 Rust 发布工作流中的徽章显示格式

### Changed
- 更新 README 文件，添加徽章以增强可视化信息
- 更新 docs 依赖项版本

### Fixed
- 修改鉴权函数以返回简化的 404 响应体
- 更新获取标签逻辑以支持当前标签和上一个标签之间的提交信息
- 更新 GitHub 容器注册表地址以指向正确的仓库
- 更新 Dockerfile 以支持 musl 目标的静态链接编译
- 更新 README 文件中的 Docker 镜像版本和 GitHub 工作流状态徽章
- 处理 Docker 容器重启时的 PID 文件清理逻辑
- 移除不必要的 RwLock 导入，简化存储模块
- 更新文档中的分支名称，从 main 修改为 master

### Style
- 在 print_usage 宏中添加分号

### Refactored
- 重构存储系统实现

## [v0.0.5] - 2025-05-29

### Added
- 添加 Admin API 支持，提供短链接管理的 HTTP 接口
- 添加 VitePress 文档部署工作流
- add VitePress documentation site
- 更新 Admin API 鉴权逻辑，禁用时返回 404，添加相关文档说明
- 增强鉴权函数，添加未设置管理员令牌的日志信息
- 更新 admin 路由前缀配置，支持通过环境变量设置

### Changed
- 更新文档，添加链接过期时间支持和环境变量配置示例
- 更新短链接处理逻辑，优化 HTTP 响应头和日志记录
- 更新 Docker 构建流程，优化文件复制路径和构建命令
- 重构构建流程，合并 Linux 和 Windows 构建作业，添加 macOS 构建支持
- 更新发布流程，优化二进制文件下载链接和文档

## [v0.0.4] - 2025-05-28

### Added
- 添加文件存储和 Redis 存储支持，重构 CLI 功能
- 添加链接过期检查，更新短链接重定向逻辑
- 重构 CLI 输出，添加彩色输出宏以增强用户体验
- 添加对 Windows 系统的锁文件支持，优化进程管理

### Changed
- 更新 Windows 平台的锁文件机制，优化链接配置重载逻辑
- 更新 Docker 和 Rust CI 工作流，添加文件路径监控以支持更改检测
- 更新发布说明，简化内容并添加文档链接
- 更新依赖项

## [v0.0.3] - 2025-05-28

### Added
- 添加 PID 文件管理功能，防止服务器重复启动 (fix #1)
- 添加短链接存储和加载功能，重构信号通知机制
- 添加短链 worker 模块并改进响应处理
- 添加变更日志生成步骤，更新发布说明文档
- 添加 workflow_dispatch 触发器，优化 Docker CI 工作流

### Changed
- 优化随机短码生成
- 更新 Rust CI 工作流，添加 i686 目标支持并优化发布信息
- 更新 Dockerfile，修复 links.json 复制路径并简化构建命令
- Create CNAME

## [v0.0.2] - 2025-05-27

### Added
- 添加随机短码生成机制，支持短码覆盖选项
- 添加 Docker 支持，创建 Dockerfile 和相关配置文件
- 添加 Unix 和 Windows 平台的信号与文件监听机制以支持动态重新加载配置
- 添加 Docker CI 工作流，支持自动构建和推送 Docker 镜像

### Changed
- 更新 README，增强项目亮点和使用说明，添加 Docker 部署示例
- 更新 Docker CI 工作流，修正镜像名称为短链接服务
- 优化交叉编译逻辑

## [v0.0.1] - 2025-05-27

### Added
- 实现基本短链接服务
- 更新短链接服务，添加链接管理功能和 PID 文件支持

## [v0.0.0] - 2025-05-26

### Added
- 初始化 shortlinker rust 项目
- 设置 CI/CD 和交叉编译
- 添加权限配置并优化发布目录创建步骤
- Update README.md
- Initial commit

[Unreleased]: https://github.com/AptS-1547/shortlinker/compare/v0.3.0-alpha.3...HEAD
[v0.3.0-alpha.3]: https://github.com/AptS-1547/shortlinker/compare/v0.3.0-alpha.2...v0.3.0-alpha.3
[v0.3.0-alpha.2]: https://github.com/AptS-1547/shortlinker/compare/v0.3.0-alpha.1...v0.3.0-alpha.2
[v0.3.0-alpha.1]: https://github.com/AptS-1547/shortlinker/compare/v0.2.3-alpha.3...v0.3.0-alpha.1
[v0.2.3-alpha.3]: https://github.com/AptS-1547/shortlinker/compare/v0.2.3-alpha.2...v0.2.3-alpha.3
[v0.2.3-alpha.2]: https://github.com/AptS-1547/shortlinker/compare/v0.2.3-alpha.1...v0.2.3-alpha.2
[v0.2.3-alpha.1]: https://github.com/AptS-1547/shortlinker/compare/v0.2.2...v0.2.3-alpha.1
[v0.2.2]: https://github.com/AptS-1547/shortlinker/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/AptS-1547/shortlinker/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.7...v0.2.0
[v0.1.7-alpha.7]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.6...v0.1.7-alpha.7
[v0.1.7-alpha.6]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.5...v0.1.7-alpha.6
[v0.1.7-alpha.5]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.4...v0.1.7-alpha.5
[v0.1.7-alpha.4]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.3...v0.1.7-alpha.4
[v0.1.7-alpha.3]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.2...v0.1.7-alpha.3
[v0.1.7-alpha.2]: https://github.com/AptS-1547/shortlinker/compare/v0.1.7-alpha.1...v0.1.7-alpha.2
[v0.1.7-alpha.1]: https://github.com/AptS-1547/shortlinker/compare/v0.1.6...v0.1.7-alpha.1
[v0.1.6]: https://github.com/AptS-1547/shortlinker/compare/v0.1.5...v0.1.6
[v0.1.5]: https://github.com/AptS-1547/shortlinker/compare/v0.1.4...v0.1.5
[v0.1.4]: https://github.com/AptS-1547/shortlinker/compare/v0.1.3...v0.1.4
[v0.1.3]: https://github.com/AptS-1547/shortlinker/compare/v0.1.2...v0.1.3
[v0.1.2]: https://github.com/AptS-1547/shortlinker/compare/v0.1.1...v0.1.2
[v0.1.1]: https://github.com/AptS-1547/shortlinker/compare/v0.1.0...v0.1.1
[v0.1.0]: https://github.com/AptS-1547/shortlinker/compare/v0.0.6...v0.1.0
[v0.0.6]: https://github.com/AptS-1547/shortlinker/compare/v0.0.5...v0.0.6
[v0.0.5]: https://github.com/AptS-1547/shortlinker/compare/v0.0.4...v0.0.5
[v0.0.4]: https://github.com/AptS-1547/shortlinker/compare/v0.0.3...v0.0.4
[v0.0.3]: https://github.com/AptS-1547/shortlinker/compare/v0.0.2...v0.0.3
[v0.0.2]: https://github.com/AptS-1547/shortlinker/compare/v0.0.1...v0.0.2
[v0.0.1]: https://github.com/AptS-1547/shortlinker/compare/v0.0.0...v0.0.1
[v0.0.0]: https://github.com/AptS-1547/shortlinker/releases/tag/v0.0.0
