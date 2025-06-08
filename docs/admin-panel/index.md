# Web 管理界面

Shortlinker 提供了基于 Vue 3 的可选 Web 管理界面，位于 `admin-panel` 目录，可通过 Admin API 进行图形化管理。

要在 Shortlinker 中启用该界面，需先构建 `admin-panel/dist` 并设置 `ENABLE_FRONTEND_ROUTES=true`（同时需要 `ADMIN_TOKEN`）。该特性尚在开发中，可能不够稳定。

## 主要功能

- 🔑 Token 认证登录
- ✨ 短链接的增删改查
- 📊 实时统计与健康状态展示
- 🕒 过期时间设置与提醒

## 本地启动

```bash
cd admin-panel
yarn install
yarn dev
```

默认会在 `http://localhost:5173` 启动前端开发服务器，需要在 `.env` 或环境变量中配置 `VITE_API_URL` 指向后端接口地址。

## 构建与部署

```bash
# 构建静态文件
yarn build

# 生成的文件位于 dist/，可直接放置在任意静态服务器上
```
