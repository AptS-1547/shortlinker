# 开发指南

本文档介绍如何进行 Web 管理界面的本地开发、构建部署和贡献代码。

## 本地开发

```bash
cd admin-panel

# 安装依赖
bun install

# 启动开发服务器
bun dev
```

开发服务器会在 `http://localhost:5173` 启动。

## 环境配置

创建 `.env.local` 文件配置后端 API 地址：

```bash
# .env.local
VITE_API_URL=http://localhost:8080
```

可用的环境变量：

| 变量                   | 说明            | 默认值                   |
| ---------------------- | --------------- | ------------------------ |
| `VITE_API_URL`         | 后端 API 地址   | `http://localhost:8080`  |
| `VITE_DEFAULT_LOCALE`  | 默认语言        | `zh`                     |

## 构建部署

```bash
# 构建生产版本
bun run build

# 预览构建结果
bun run preview

# 代码检查
bun run lint
```

构建产物位于 `dist/` 目录，可以：

1. 通过 Shortlinker 内置服务（运行时配置 `features.enable_admin_panel=true`，需要重启生效）
2. 部署到独立的静态服务器（Nginx、Caddy 等）
3. 部署到 CDN（需配置 CORS）

## Docker 集成部署

如果使用 Docker，可以在构建镜像时包含前端资源：

```dockerfile
# 多阶段构建示例
FROM node:24-alpine AS frontend-builder
RUN npm install -g bun@latest
WORKDIR /app/admin-panel
COPY admin-panel/ ./
RUN bun install --frozen-lockfile
RUN bun run build

FROM rust:1.94-slim AS backend-builder
# ... Rust 构建步骤（使用 musl 静态链接）...

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/shortlinker /shortlinker
# ... 其他配置 ...
```

:::tip 提示
完整的 Dockerfile 请参考项目根目录的 `Dockerfile` 文件。官方镜像使用 `scratch` 作为基础镜像，通过 musl 静态链接实现最小化部署。
:::

## 技术栈

- **框架**：React 19 + TypeScript
- **构建工具**：Vite 8
- **路由**：React Router 7
- **状态管理**：Zustand
- **UI 组件**：Radix UI + Tailwind CSS
- **HTTP 客户端**：Axios
- **国际化**：react-i18next
- **表单验证**：React Hook Form + Zod
- **代码规范**：Biome

## 项目结构

```text
admin-panel/
├── src/
│   ├── components/     # UI 组件
│   │   ├── ui/         # 基础组件（Button、Dialog 等）
│   │   ├── layout/     # 布局组件
│   │   ├── links/      # 链接管理组件
│   │   └── settings/   # 设置页面组件
│   ├── pages/          # 页面组件
│   ├── hooks/          # 自定义 Hooks
│   ├── stores/         # Zustand 状态管理
│   ├── services/       # API 服务层
│   ├── i18n/           # 国际化配置
│   ├── router/         # 路由配置
│   ├── schemas/        # Zod 验证模式
│   ├── types/          # TypeScript 类型定义
│   └── utils/          # 工具函数
├── public/             # 静态资源
└── dist/               # 构建产物
```

## 贡献指南

欢迎提交 PR 改进 Web 管理界面！开发前请：

1. Fork 项目并创建功能分支
2. 遵循现有代码风格（使用 Biome）
3. 添加必要的类型定义
4. 确保构建通过：`bun run lint && bun run build`
5. 提交 PR 并描述改动内容

## 相关链接

- 📖 [Web 管理界面概览](./index)
- ❓ [故障排除](./troubleshooting)
