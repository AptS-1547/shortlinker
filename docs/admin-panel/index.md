# Web 管理界面

:::warning v0.3.x 版本提醒
当前版本 (v0.3.x) 正在进行大幅度功能调整和重构，更新频率较高。建议：
- 📌 生产环境请使用稳定版本标签
- 🔄 开发环境可跟随最新版本体验新功能
- 📖 文档可能滞后于代码实现，以实际功能为准
:::

Shortlinker 提供了基于 Vue 3 + TypeScript 的现代化 Web 管理界面，位于 `admin-panel` 目录，通过 Admin API 提供完整的图形化管理能力。

## 启用方式

要在 Shortlinker 中启用 Web 管理界面：

1. **构建前端资源**：
   ```bash
   cd admin-panel
   yarn install
   yarn build
   ```

2. **配置环境变量**：
   ```bash
   ENABLE_ADMIN_PANEL=true
   ADMIN_TOKEN=your_secure_admin_token
   FRONTEND_ROUTE_PREFIX=/panel  # 可选，默认为 /panel
   ```

3. **访问界面**：
   启动 Shortlinker 后访问 `http://your-domain:8080/panel`

:::tip 提示
该特性为**实验性功能**，目前处于活跃开发阶段。如遇问题请通过 GitHub Issues 反馈。
:::

## 自定义前端

Shortlinker 支持使用自定义前端实现。你可以通过将自定义前端放在 `./frontend-panel` 目录来替换内置的管理面板。

### 使用方法

1. **准备你的前端**：
   - 构建你的前端应用
   - 将构建产物放在项目根目录的 `./frontend-panel` 目录下
   - 确保 `index.html` 在该目录的根目录

2. **模版仓库**：
   - 官方模版：[shortlinker-frontend](https://github.com/AptS-1547/shortlinker-frontend/)
   - Fork 后根据需求自定义

3. **参数注入**：
   HTML 文件（`index.html`、`manifest.webmanifest`）中的以下占位符会被自动替换：
   - `%BASE_PATH%` - 前端路由前缀（如 `/panel`）
   - `%ADMIN_ROUTE_PREFIX%` - Admin API 前缀（如 `/admin`）
   - `%HEALTH_ROUTE_PREFIX%` - Health API 前缀（如 `/health`）
   - `%SHORTLINKER_VERSION%` - 当前 Shortlinker 版本

4. **检测**：
   Shortlinker 启动时会自动检测 `./frontend-panel` 目录，如果存在则使用它。你会看到日志：
   ```
   Custom frontend detected at: ./frontend-panel
   ```

:::warning 优先级
自定义前端优先级高于内置管理面板。如果 `./frontend-panel` 存在，将使用它而不是嵌入的前端。
:::

## 主要功能

### 核心功能
- 🔑 **Token 认证登录**：基于 Bearer Token 的安全认证
- 📋 **链接管理**：完整的 CRUD 操作界面
  - 创建新短链接（支持自定义短码、过期时间、密码保护）
  - 编辑现有链接
  - 删除链接（带确认提示）
  - 批量操作（计划中）
- 📊 **数据可视化**：
  - Dashboard 仪表板展示关键指标
  - 点击统计图表
  - 存储后端状态监控
- 🔍 **高级功能**：
  - 链接筛选和搜索（活跃/过期/受保护）
  - 分页浏览
  - 过期时间提醒
  - 复制短链接到剪贴板
  - 二维码生成（计划中）

### 界面特性
- 🌓 **主题切换**：明暗主题支持
- 🌍 **国际化**：中英文双语界面
- 📱 **响应式设计**：适配桌面和移动端
- ⚡ **性能优化**：Vue 3 Composition API + Vite 构建

## 开发指南

### 本地开发

```bash
cd admin-panel

# 安装依赖
yarn install

# 启动开发服务器
yarn dev
```

开发服务器会在 `http://localhost:5173` 启动。

### 环境配置

创建 `.env.local` 文件配置后端 API 地址：

```bash
# .env.local
VITE_API_URL=http://localhost:8080
```

可用的环境变量：

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `VITE_API_URL` | 后端 API 地址 | `http://localhost:8080` |
| `VITE_DEFAULT_LOCALE` | 默认语言 | `zh` |

### 构建部署

```bash
# 构建生产版本
yarn build

# 预览构建结果
yarn preview

# 类型检查
yarn type-check

# 代码检查
yarn lint
```

构建产物位于 `dist/` 目录，可以：
1. 通过 Shortlinker 内置服务（设置 `ENABLE_ADMIN_PANEL=true`）
2. 部署到独立的静态服务器（Nginx、Caddy 等）
3. 部署到 CDN（需配置 CORS）

### Docker 集成部署

如果使用 Docker，可以在构建镜像时包含前端资源：

```dockerfile
# 多阶段构建示例
FROM node:18 AS frontend-builder
WORKDIR /app/admin-panel
COPY admin-panel/package.json admin-panel/yarn.lock ./
RUN yarn install
COPY admin-panel/ ./
RUN yarn build

FROM rust:1.75 AS backend-builder
# ... Rust 构建步骤 ...

FROM debian:bookworm-slim
COPY --from=frontend-builder /app/admin-panel/dist /app/admin-panel/dist
# ... 其他配置 ...
```

## 技术栈

- **框架**：Vue 3 + TypeScript
- **构建工具**：Vite 5
- **路由**：Vue Router 4
- **状态管理**：Pinia
- **UI 样式**：原生 CSS + CSS Variables
- **HTTP 客户端**：Axios
- **图表**：Chart.js（计划中）
- **国际化**：Vue I18n

## 界面预览

### Dashboard 仪表板
- 总链接数统计
- 活跃/过期链接数
- 点击数汇总
- 存储后端信息
- 服务运行时间

### 链接管理页面
- 表格视图展示所有链接
- 状态标签（活跃/过期/受保护）
- 点击数实时显示
- 快捷操作按钮（编辑/删除/复制）
- 分页导航

### 分析页面（计划中）
- 点击趋势图表
- 热门链接排行
- 访问来源统计

## 安全建议

1. **强密码**：使用足够复杂的 `ADMIN_TOKEN`
2. **HTTPS**：生产环境必须启用 HTTPS
3. **路径隔离**：考虑使用非默认的 `FRONTEND_ROUTE_PREFIX`
4. **网络隔离**：仅在受信任网络中暴露管理界面
5. **定期更新**：及时更新依赖包修复安全漏洞

## 故障排除

### 登录失败

```bash
# 检查 ADMIN_TOKEN 是否正确配置
echo $ADMIN_TOKEN

# 检查 API 地址配置
cat admin-panel/.env.local

# 查看浏览器控制台错误
```

### 构建失败

```bash
# 清理依赖重新安装
rm -rf node_modules yarn.lock
yarn install

# 检查 Node.js 版本（需要 >= 18）
node --version
```

### 样式异常

```bash
# 清理 Vite 缓存
rm -rf admin-panel/.vite
yarn dev
```

## 开发路线图

- ✅ 基础 CRUD 功能
- ✅ 认证和权限
- ✅ 主题切换
- ✅ 国际化支持
- 🚧 批量操作
- 🚧 二维码生成
- 🚧 点击统计图表
- 📋 导出/导入功能
- 📋 链接分组管理
- 📋 自定义域名支持

## 贡献指南

欢迎提交 PR 改进 Web 管理界面！开发前请：

1. Fork 项目并创建功能分支
2. 遵循现有代码风格（使用 ESLint + Prettier）
3. 添加必要的类型定义
4. 确保构建通过：`yarn type-check && yarn build`
5. 提交 PR 并描述改动内容

## 相关链接

- 📖 [Admin API 文档](/api/admin)
- 🔧 [环境变量配置](/config/)
- 🚀 [部署指南](/deployment/)
