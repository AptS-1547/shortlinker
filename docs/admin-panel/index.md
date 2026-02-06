# Web 管理界面

Shortlinker 提供了基于 React 19 + TypeScript 的现代化 Web 管理界面，位于 `admin-panel` 目录，通过 Admin API 提供完整的图形化管理能力。

## 3 分钟上手

1. 先按 [启用方式](#启用方式) 打开面板
2. 用管理员密码登录（`api.admin_token`）
3. 按“创建链接 → 列表筛选 → 导出/导入”完成日常操作

> 如需 API 自动化，请直接跳到 [Admin API 文档](/api/admin)。

## 启用方式

要在 Shortlinker 中启用 Web 管理界面：

1. **构建前端资源**：

   ```bash
   cd admin-panel
   bun install
   bun run build
   ```

2. **启用配置（运行时配置）**：

   ```bash
   # 启用管理面板（运行时配置，写入数据库；需要重启生效）
   ./shortlinker config set features.enable_admin_panel true

   # 可选：修改前端路由前缀（需要重启）
   ./shortlinker config set routes.frontend_prefix /panel
   ```

3. **访问界面**：
   启动 Shortlinker 后访问 `http://your-domain:8080/panel`

> 提示：
> - 管理员登录密码是运行时配置 `api.admin_token` 的明文值；首次启动会生成并写入 `admin_token.txt`（若文件不存在），也可用 `./shortlinker reset-password` 重置。
> - `routes.frontend_prefix` / `routes.admin_prefix` / `routes.health_prefix` 等路由前缀配置修改后需要重启生效。

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

   ```text
   Custom frontend detected at: ./frontend-panel
   ```

:::warning 优先级
自定义前端优先级高于内置管理面板。如果 `./frontend-panel` 存在，将使用它而不是嵌入的前端。
:::

## 主要功能

### 核心能力

- 🔑 **登录与会话认证**：使用 `api.admin_token` 登录，后端通过 Cookie 提供会话认证
- 📋 **链接管理**：支持创建、编辑、删除、批量删除、二维码生成
- 🔍 **检索与筛选**：支持关键字搜索、状态筛选、时间范围筛选、多列排序、分页
- 📥 **导入导出**：CSV 导入/导出，支持冲突策略与拖拽上传
- ⚙️ **设置中心**：运行时配置编辑、配置历史、配置重载

### 界面能力

- 🌓 主题切换（浅色/深色/跟随系统）
- 🌍 国际化（中文、英文、日语、法语、俄语）
- 📱 响应式布局（桌面与移动端）
- 📲 PWA 安装与离线访问

## 界面预览（快速了解）

### Dashboard 仪表板

- 展示总链接数、活跃/过期比例、点击数据
- 展示存储后端信息与系统运行时间

### 链接管理页面

- 提供表格视图、状态标签与快捷操作按钮
- 提供筛选、排序、分页与列配置

### 设置页面

- 偏好设置：主题与语言
- 系统设置：运行时配置管理与重载

### 数据分析页面（开发中）

- 计划提供点击趋势、热门链接、访问来源等统计视图

## 开发路线图（简要）

- ✅ 基础 CRUD、认证、主题切换、国际化
- ✅ 批量操作、二维码、导入导出、PWA
- ✅ 系统配置管理
- 🚧 点击统计图表
- 📋 链接分组管理与自定义域名支持

## 相关链接

- 📖 [Admin API 文档](/api/admin)
- 🔧 [配置指南](/config/)
- 🚀 [部署指南](/deployment/)
- 🛠️ [开发指南](./development)
- ❓ [故障排除](./troubleshooting)
