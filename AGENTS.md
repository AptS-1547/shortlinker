# shortlinker

shortlinker 是一个使用 Rust 和 Actix-web 构建的短链接服务。它提供 HTTP 307 重定向、短码和目标 URL 管理、密码保护、过期时间、点击统计、管理 API、CLI、Web 管理面板，以及 SQLite/MySQL/PostgreSQL 存储支持。

修改代码时围绕“短链接服务”的领域组织实现，不要把其他项目的文件、云盘、团队空间等概念带进来。仓库包含后端、管理面板和 VitePress 文档，改动前先确定行为属于哪一层。

## 工作前必须先看

- 先读现有代码模式，再实现。尤其要先看目标模块的调用方、错误处理和测试；看不清边界时停下来确认，不要凭感觉增加抽象。
- 修改前优先沿真实入口追链路：`src/api/` -> `src/services/` / `src/storage/`，或 `src/interfaces/cli/` / `src/client/` -> `src/services/` / `src/storage/`。
- `src/api/services/redirect.rs` 是有意保留的重定向热路径：它直接组合缓存和存储，不要为了形式统一把它改成调用 `LinkService`。
- `src/config/definitions.rs` 是配置元数据和默认值的单一来源。新增、修改配置前同时检查 schema、runtime config、持久化和管理面板使用方。
- 这个仓库可能有大量未提交或未跟踪文件。不要回滚用户改动；只修改任务相关文件，同文件有交叉改动时先读清楚再动手。
- 收到 code review 评论时，先判断是真问题还是误报；真实问题分批修复，每批都做最小的编译/测试验证。

## 项目结构

```text
src/
  api/                  Actix HTTP 路由、管理 API、中间件、重定向和前端静态资源
  analytics/            点击事件、异步写入、小时/每日汇总和保留策略
  cache/                Bloom、负缓存、对象缓存和组合缓存
  client/               IPC/内部客户端及配置、链接客户端
  config/               静态配置、运行时配置、schema、校验和默认值
  interfaces/cli/       CLI 命令实现（链接、配置、导入导出、状态等）
  metrics/               Prometheus 实现（需要 `metrics` feature）
  runtime/              启动、关闭、生命周期和 server mode
  services/             链接、配置、分析、GeoIP 等业务编排
  storage/              SeaORM 存储、数据库连接、查询、写入和配置存储
  system/               日志、panic handler、平台和进程相关能力
  utils/                URL、短码、密码、数字转换、时间等通用工具
  main.rs / lib.rs      二进制入口和公共库入口
migration/              SeaORM migration crate 和实体定义
tests/                  Rust 集成测试（重定向、管理 API、存储、CLI、IPC、分析等）
benches/                Criterion 基准测试
admin-panel/            React 19 + Vite + TypeScript 管理面板
docs/                   VitePress 用户、部署、配置、API 和开发文档
assets/                 README 和管理面板使用的静态资源
.github/workflows/      Rust、前端、文档、审计和发布流水线
```

## 技术栈

- 后端：Rust 2024，最低 Rust 版本以 `Cargo.toml` 的 `rust-version` 为准（当前为 1.88），Actix-web、Tokio、SeaORM、Serde、Tracing。
- 数据库：SQLite 默认，兼容 MySQL 和 PostgreSQL；数据库类型由 database URL 推断。
- 缓存：Bloom existence filter、negative cache、object cache 和 composite cache；重定向路径依赖这条查询链。
- 认证：管理 API 支持 Bearer Token/JWT 和 Cookie，会话刷新、CSRF 与登录/刷新限流由现有中间件负责。
- 分析：点击计数通过异步 manager/channel 写入，详细日志、GeoIP、小时/每日 rollup 受运行时配置控制。
- 前端：React 19、Vite、TypeScript、Tailwind CSS、Radix UI、Zustand、i18next、Vitest、Biome；包管理器使用 Bun。
- 文档：VitePress，包管理器使用 Bun。
- 可选 feature：`server`、`cli`、`metrics`、`openapi`、`full`；默认启用 `server` 和 `cli`。

## 开发命令

```bash
# Rust 基础检查
cargo fmt --all -- --check
cargo check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 测试：开发时优先缩小范围
cargo test --lib <test_filter>
cargo test --test <test_file> <test_filter>
cargo test --test redirect_tests
cargo test --test admin_api_tests
cargo test

# OpenAPI 和管理面板类型生成
cargo test --features openapi --test generate_openapi
cd admin-panel && bun run generate-api && cd ..

# 运行服务或 CLI
cargo run
cargo run -- add <code> <target>
cargo run -- list
cargo run -- config generate

# 管理面板
cd admin-panel
bun install --frozen-lockfile --registry https://registry.npmjs.org
bun run dev
bun run check
bun run test
bun run test:coverage
bun run build

# 文档
cd docs
bun install --frozen-lockfile --registry https://registry.npmjs.org
bun run docs:dev
bun run docs:build
```

Rust CI 的完整约束是 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings` 和 workspace 测试。改动较小时先跑目标测试，再跑 `cargo check`；改动跨模块或准备提交时再跑完整检查。不要在每次小改动时无目标地反复编译全部 workspace。

## 架构和边界

### HTTP API

- 普通管理 API 入口在 `src/api/services/admin/`，路由组合在 `routes.rs`，handler 负责鉴权、参数提取、调用业务层和响应映射。
- 链接的业务编排放在 `src/services/link_service.rs`，共享给 HTTP 管理 API 和 IPC/CLI；不要在多个入口重复实现创建、更新、删除、导入校验和密码处理。
- 数据库访问放在 `src/storage/`，尤其是 `SeaOrmStorage` 和 backend query/mutation 模块；handler 不要直接拼 SQL。
- `src/api/services/redirect.rs` 是特殊边界：重定向请求需要 Bloom -> negative cache -> object cache -> storage 的热路径，并异步更新点击统计。不要把这条路径迁移到管理 CRUD service，也不要在其中增加阻塞式分析或不必要的数据库查询。
- 前端静态资源由 `src/api/services/frontend.rs` 提供，内置面板构建产物通过 `rust-embed` 嵌入；修改资源路由时同时检查 SPA fallback、健康检查和 API 路由优先级。

### 响应和错误

- 管理 API 使用 `src/api/services/admin/helpers.rs` 的统一 JSON 响应和 `ErrorCode`；新 handler 应复用 `success_response`、`error_response`、`api_result` 或现有等价封装。
- 重定向、健康检查、metrics、静态资源和 CLI 不强行套管理 API envelope：307、404/500、Prometheus text 和文件响应必须保持客户端兼容。
- 领域错误统一使用 `ShortlinkerError`/`src/errors.rs` 的现有转换路径，不要在 handler 中散落字符串错误和不一致的状态码。
- 新增或修改 endpoint 时同步检查认证、CSRF/CORS、限流、缓存头和日志字段。
- 管理 API handler 使用 `aster_forge_api_docs_macros::path` 声明 OpenAPI 元数据；schema 聚合在 `src/api/openapi.rs`。修改 DTO 或 endpoint 后依次运行 OpenAPI 生成测试和管理面板 `generate-api`，业务代码从 `admin-panel/src/services/types.ts` 导入类型，不直接依赖生成文件。

### 配置

- 所有可配置项目的 key、类型、默认值、敏感标记、可编辑性和 action 在 `src/config/definitions.rs` 注册。
- 运行时读取使用现有 `get_config()`、`get_runtime_config()` 和 `keys`，不要在业务模块写重复默认值或直接读取环境变量绕过配置层。
- 配置变更若需要持久化、历史记录、热重载或前端编辑，必须同时检查 `config_store`、schema、admin config API、配置校验和管理面板。
- 敏感值（admin token、JWT secret、密码、cookie secret 等）不能出现在日志、错误消息、导出文件或测试输出中；导入/导出时遵守现有脱敏和哈希处理。

### 存储、缓存和分析

- 新增表或修改字段时，同时更新 `migration/` 的 migration/entity，并补 SQLite 集成测试；涉及数据库方言的 SQL 时检查 MySQL/PostgreSQL 分支。
- 使用 `StorageFactory`、`SeaOrmStorage` 和现有 storage API，不在 service 或 handler 中直接依赖具体 SeaORM entity 进行跨层写入。
- 缓存失效必须和链接创建、更新、删除、导入、过期处理保持一致；新增读取路径要明确是否需要 Bloom、negative cache 或 object cache。
- 点击统计走现有 `analytics` manager/channel。重定向响应不能等待详细日志、GeoIP 或 rollup；失败要通过 tracing 记录，不要静默丢弃错误。
- 过期链接的缓存 TTL 必须尊重 `ShortLink::cache_ttl` 语义，不能把已经过期的链接重新缓存。

## Rust 代码约定

- 遵循 rustfmt 和 Clippy；公共类型和复杂协议边界添加必要的文档注释。
- 业务函数优先保持“解析/校验 -> 加载上下文 -> 调用 storage -> 更新缓存/分析 -> 返回结果”的清晰顺序，避免把所有逻辑塞进 handler。
- 异步 fire-and-forget 操作使用现有 tracing 记录错误，不要用静默的 `let _ =` 掩盖失败。
- 不要为了抽象而抽象：先复用现有 trait、storage、cache、service 和错误类型；只有存在真实共享行为时才新增模块。
- URL、短码、过期时间、密码、批量导入字段和配置值必须在 service/DTO 边界校验；路径、URL 和 redirect target 处理要覆盖空值、非法格式和边界长度。
- 数字转换、时间解析、密码处理和 URL 校验优先使用 `src/utils/` 现有 helper，避免重复实现。
- 数据库事务和缓存失效顺序要明确。涉及链接元数据、点击计数、配置历史或批量操作时，测试成功和失败路径。

## 前端约定

具体前端修改先读取 `admin-panel/` 当前组件和 store 的模式；根目录约束如下：

- API 调用使用现有 service/store 封装，不在组件内重复拼 axios 请求、认证 header 或错误映射。
- 后端配置 schema 和 API 返回值是能力真相；不要在前端复制一套配置 key、默认值或后端校验规则。
- 使用 TypeScript 类型和现有 Zod schema；不要用 `any` 绕过类型检查，也不要手写与后端响应冲突的接口。
- 保持 React hooks、路由、i18next 和 Zustand 的现有组织方式；新增可见文案放入对应 locale，不要在组件里散落硬编码。
- 使用项目已有的 Radix/UI 和图标封装，保持管理界面的密度、键盘操作、加载/空态/错误态一致。
- 前端改动至少运行 `bun run check`、相关 Vitest 测试和 `bun run build`；构建产物 `admin-panel/dist` 是否需要更新以任务要求和发布流程为准。

## 测试要求

- 新增后端行为至少补单元测试或集成测试；短码校验、307 重定向、过期、密码、缓存失效、认证、CSRF、批量操作、导入导出和分析边界不能只靠手工验证。
- 修改 storage、migration 或 SQL 时至少跑相关 SQLite 测试，并检查 MySQL/PostgreSQL 的方言分支和错误处理。
- 修改 redirect 热路径时覆盖 cache hit、cache miss、过期、Bloom false positive、非法短码、UTM 透传和数据库错误响应。
- 修改管理 API 时覆盖成功响应、错误码、鉴权失败、输入校验、分页/批量边界和敏感字段脱敏。
- 修改配置时覆盖默认值、schema、运行时热重载、需要重启的标记和敏感配置处理。
- 修改前端 service、store、表单或关键交互时补 Vitest；涉及完整页面流程时至少运行构建并手动检查开发服务器行为。
- 修改文档只需运行 `cd docs && bun run docs:build`；文档构建失败不能被代码测试掩盖。

## 文档、迁移和发布

- 用户可见行为修改时更新直接相关的 `README`、`docs/` 或 `CHANGELOG.md`，不要借机做无关的大规模改写。
- 文档同时存在英文和中文入口，新增重要配置/API/部署行为时检查两种语言的导航和链接是否仍然有效。
- migration 必须保持可重复执行和正确顺序；不要修改已经发布 migration 的历史语义，新增变更使用新的 migration 文件。
- 发布由 `v*` tag 触发 GitHub Actions，涉及管理面板时先构建 `admin-panel/dist`，涉及镜像或跨平台二进制时检查对应 workflow 和 Dockerfile。
- 保留 MIT 许可证约束，不复制许可证不兼容的代码或资源。

## Git 和 Review

- 只提交任务相关改动；不要把 `target/`、coverage、临时数据库、日志或本地环境文件带进提交。
- 提交信息遵循 Conventional Commits，例如 `feat(api): ...`、`fix(redirect): ...`、`test(storage): ...`、`docs: ...`。
- review 时按严重性报告真实问题，给出文件和行号；已修复或与当前代码不匹配的评论要明确标记，不要为了满足机器人而引入无效改动。
- 修复 review 问题时按批次验证：每批先跑针对性 `cargo check`/测试或前端 check，再继续下一批；最后补一次与改动范围匹配的完整检查。

## 参考资料

- 项目说明：`README.md`、`README.zh.md`
- 贡献流程：`CONTRIBUTING.md`
- 配置示例：`config.example.toml`
- 用户和部署文档：`docs/`
- 管理面板：`admin-panel/`
- 数据库 migration：`migration/`
- CI 约束：`.github/workflows/rust.yml`、`.github/workflows/admin-panel.yml`、`.github/workflows/docs-check.yml`
