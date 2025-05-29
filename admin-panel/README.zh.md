# Shortlinker Admin Panel

一个现代化的 Web 管理界面，用于管理 [Shortlinker](../README.zh.md) 短链接服务。

## 功能特性

- 🎨 **现代化界面** - 基于 Vue 3 + TailwindUI 的响应式设计
- 🔐 **安全认证** - Bearer Token 认证，与后端 Admin API 无缝集成
- 📊 **完整管理** - 支持短链接的增删改查操作
- ⚡ **实时更新** - 操作后自动刷新数据
- 🕐 **过期管理** - 可视化过期时间设置和显示

## 技术栈

- **前端框架**: Vue 3 + TypeScript
- **UI 组件**: TailwindUI + Headless UI
- **状态管理**: Pinia
- **构建工具**: Vite
- **包管理**: Yarn

## 开发状态

🚧 **正在开发中** - 该管理面板目前还在规划和开发阶段，将在未来版本中完成。

### 计划功能

- [ ] 用户认证界面
- [ ] 短链接列表管理
- [ ] 创建和编辑短链接
- [ ] 批量操作功能
- [ ] 统计数据展示
- [ ] 国际化支持

## 环境配置

未来将支持以下环境变量配置：

```bash
# Shortlinker 服务地址
VITE_API_BASE_URL=http://localhost:8080

# Admin API 路由前缀
VITE_ADMIN_ROUTE_PREFIX=/admin

# 默认管理员 Token（开发环境）
VITE_DEFAULT_ADMIN_TOKEN=your_admin_token
```

## API 集成

Admin Panel 将基于 Shortlinker 的 [Admin API](../src/admin.rs) 构建，支持以下接口：

- `GET /admin/link` - 获取所有短链接
- `POST /admin/link` - 创建新短链接  
- `GET /admin/link/{code}` - 获取指定短链接
- `PUT /admin/link/{code}` - 更新短链接
- `DELETE /admin/link/{code}` - 删除短链接

## 认证方式

所有 API 请求都需要在 Header 中包含 Bearer Token：

```
Authorization: Bearer {ADMIN_TOKEN}
```

## 开发计划

该管理面板将在后续版本中逐步实现，敬请期待。

## 相关文档

- 📖 [Shortlinker 主文档](../README.zh.md)
- 🔧 [Admin API 源码](../src/admin.rs)
- ⚙️ [配置说明](../docs/config/index.md)

## 许可证

MIT License - 查看 [LICENSE](../LICENSE) 文件了解详细信息。