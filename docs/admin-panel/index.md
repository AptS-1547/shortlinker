# Web 管理界面

:::warning v0.3.x 版本提醒
当前版本 (v0.3.x) 正在进行大幅度功能调整和重构，更新频率较高。建议：

- 📌 生产环境请使用稳定版本标签
- 🔄 开发环境可跟随最新版本体验新功能
- 📖 文档可能滞后于代码实现，以实际功能为准
:::

Shortlinker 提供了基于 React 19 + TypeScript 的现代化 Web 管理界面，位于 `admin-panel` 目录，通过 Admin API 提供完整的图形化管理能力。

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

   ```text
   Custom frontend detected at: ./frontend-panel
   ```

:::warning 优先级
自定义前端优先级高于内置管理面板。如果 `./frontend-panel` 存在，将使用它而不是嵌入的前端。
:::

## 主要功能

### 核心功能

- 🔑 **登录与会话认证**：使用管理员密码（`api.admin_token`）登录，后端通过 `Set-Cookie` 下发 JWT Cookie（Access/Refresh），前端基于 Cookie 会话访问接口
- 📋 **链接管理**：完整的 CRUD 操作界面
  - 创建新短链接（支持自定义短码、过期时间、密码保护）
  - 编辑现有链接
  - 删除链接（带确认提示）
  - 批量选择和批量删除
  - 二维码生成
- 📊 **数据可视化**：
  - Dashboard 仪表板展示关键指标
  - 存储后端状态监控
  - 系统运行时间显示
- 🔍 **高级功能**：
  - 按代码或 URL 搜索
  - 按状态筛选（全部/活跃/过期）
  - 按创建时间范围筛选
  - 多列排序（代码、目标、点击数、创建时间、过期时间）
  - 分页浏览（支持 10/20/50/100 条每页）
  - 复制短链接到剪贴板
  - 列配置（显示/隐藏表格列）
- 📥 **导入/导出**：
  - CSV 格式导出（支持筛选条件）
  - CSV 格式导入（支持 skip/overwrite/error 三种冲突处理模式）
  - 拖拽上传支持

### 界面特性

- 🌓 **主题切换**：浅色/深色/跟随系统三种模式
- 🌍 **国际化**：支持 5 种语言（中文、英文、日语、法语、俄语）
- 📱 **响应式设计**：适配桌面和移动端
- ⚡ **性能优化**：React 19 + Vite 构建
- 📲 **PWA 支持**：可安装到桌面，支持离线访问

### 设置页面

- ⚙️ **偏好设置**：主题选择、语言切换
- 🔧 **系统配置**：
  - 运行时配置管理（分组显示）
  - 配置编辑（支持 string/number/boolean/json 类型）
  - 配置历史记录
  - 重载配置
- ℹ️ **关于**：版本信息、技术栈、开源协议、项目链接

## 界面预览

### Dashboard 仪表板

- 总链接数统计
- 活跃/过期链接数
- 点击数汇总
- 存储后端信息
- 系统运行时间
- 最近创建的链接列表

### 链接管理页面

- 表格视图展示所有链接
- 状态标签（活跃/过期/受保护）
- 点击数实时显示
- 快捷操作按钮（编辑/删除/复制/二维码）
- 批量选择和操作
- 高级筛选栏
- 分页导航
- 列配置下拉菜单

### 设置页面

- 偏好设置标签页（主题/语言）
- 系统配置标签页（运行时配置管理）
- 关于标签页（版本/技术栈/链接）

### 数据分析页面（开发中）

- 点击趋势图表
- 热门链接排行
- 访问来源统计

## 开发路线图

- ✅ 基础 CRUD 功能
- ✅ 认证和权限
- ✅ 主题切换
- ✅ 国际化支持（5 种语言）
- ✅ 批量操作
- ✅ 二维码生成
- ✅ 导入/导出功能
- ✅ PWA 支持
- ✅ 系统配置管理
- 🚧 点击统计图表
- 📋 链接分组管理
- 📋 自定义域名支持

## 相关链接

- 📖 [Admin API 文档](/api/admin)
- 🔧 [配置指南](/config/)
- 🚀 [部署指南](/deployment/)
- 🛠️ [开发指南](./development)
- ❓ [故障排除](./troubleshooting)
