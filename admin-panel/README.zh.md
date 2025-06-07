# Shortlinker Admin Panel

一个现代化的 Web 管理界面，用于管理 [Shortlinker](../README.zh.md) 短链接服务。

要在 Shortlinker 中启用此界面，需要先构建 `dist` 目录并设置 `ENABLE_FRONTEND_ROUTES=true`，同时保证已配置 `ADMIN_TOKEN`。该功能仍处于试验阶段，可能不稳定。
## 功能特性

- 🎨 **现代化界面** - 基于 Vue 3 + Vite 的响应式设计
- 🔐 **安全认证** - Token 认证访问 Admin API
- 📊 **完整管理** - 支持短链接的增删改查操作
- ⚡ **实时更新** - 操作后自动刷新数据
- 🕐 **过期管理** - 可视化过期时间设置和显示（支持本地时区）
- 💚 **健康监控** - 实时显示后端服务状态
- 🔄 **客户端路由** - 基于 Vue Router 的单页应用路由系统

## 技术栈

- **前端框架**: Vue 3 + TypeScript + Vite
- **UI 组件**: TailwindCSS + Vue
- **HTTP 客户端**: Axios
- **状态管理**: Pinia
- **路由系统**: Vue Router
- **包管理**: Yarn

## 已完成功能

- ✅ 用户认证界面
- ✅ 短链接列表管理
- ✅ 创建和编辑短链接
- ✅ 删除链接功能
- ✅ 健康状态监控
- ✅ 本地时区时间显示
- ✅ 客户端路由系统
- ✅ 冲突检测和处理

## 环境配置

支持以下环境变量配置：

```bash
# Shortlinker 服务地址
VITE_API_BASE_URL=http://localhost:8080

# Admin API 路由前缀
VITE_ADMIN_ROUTE_PREFIX=/admin

# 健康检查路由前缀
VITE_HEALTH_ROUTE_PREFIX=/health
```

## API 集成

Admin Panel 基于 Shortlinker 的 [Admin API](../src/services/admin.rs) 构建，支持以下接口：

- `GET /admin/link` - 获取所有短链接
- `POST /admin/link` - 创建新短链接  
- `GET /admin/link/{code}` - 获取指定短链接
- `PUT /admin/link/{code}` - 更新短链接
- `DELETE /admin/link/{code}` - 删除短链接
- `GET /health` - 健康检查

## 认证方式

所有 API 请求都需要在 Header 中包含 Bearer Token：

```
Authorization: Bearer {ADMIN_TOKEN}
```

## 开发运行

```bash
# 安装依赖
yarn install

# 开发模式运行
yarn dev

# 构建生产版本
yarn build
```

## 路由系统

管理面板使用 Vue Router 路由系统，支持以下路由：

- `/admin/login` - 用户登录页面
- `/admin/dashboard` - 主控制台（链接管理）
- `/admin/links` - 链接管理页面
- `/admin/analytics` - 数据分析页面（规划中）

### 路由特性

- 🔒 **路由守卫** - 自动检查认证状态
- 📱 **响应式** - 支持移动端和桌面端
- ⚡ **快速切换** - 基于 Vue Router 的即时路由
- 🔄 **状态保持** - 路由切换时保持应用状态

## 相关文档

- 📖 [Shortlinker 主文档](../README.zh.md)
- 🔧 [Admin API 源码](../src/services/admin.rs)
- ⚙️ [配置说明](../docs/config/index.md)

## 许可证

MIT License - 查看 [LICENSE](../LICENSE) 文件了解详细信息。
