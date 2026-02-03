# 故障排除

本文档介绍 Web 管理界面的常见问题和解决方案，以及安全建议。

## 常见问题

### 登录失败

```bash
# 获取/确认管理员密码（运行时配置 `api.admin_token` 的明文值）
# - 首次启动通常会生成 `admin_token.txt`（若存在）
cat admin_token.txt

# - 忘记密码可重置（写入数据库）
./shortlinker reset-password

# 检查（独立部署的）管理面板 API 地址配置
cat admin-panel/.env.local

# 查看浏览器控制台错误
```

**可能原因**：

- 管理员密码（`api.admin_token`）错误
- 后端服务未启动
- API 地址配置错误
- CORS 配置问题

### 构建失败

```bash
# 清理依赖重新安装
rm -rf node_modules bun.lock
bun install

# 检查 Bun 版本
bun --version
```

**可能原因**：

- 依赖版本冲突
- Bun 版本过低
- 网络问题导致依赖下载失败

### 样式异常

```bash
# 清理 Vite 缓存
rm -rf admin-panel/.vite
bun dev
```

**可能原因**：

- Vite 缓存过期
- Tailwind CSS 配置问题
- 浏览器缓存

### 页面空白

**可能原因**：

- JavaScript 错误，查看浏览器控制台
- 路由配置问题
- 环境变量未正确注入

### API 请求失败

**可能原因**：

- 后端服务未启动
- CORS 配置问题
- Token 过期或无效
- 网络连接问题

## 安全建议

1. **强密码**：使用足够复杂的管理员密码（`api.admin_token`）
2. **HTTPS**：生产环境必须启用 HTTPS
3. **路径隔离**：考虑使用非默认的 `routes.frontend_prefix`
4. **网络隔离**：仅在受信任网络中暴露管理界面
5. **定期更新**：及时更新依赖包修复安全漏洞

## 获取帮助

如果以上方法无法解决问题，请：

1. 查看 [GitHub Issues](https://github.com/AptS-1547/shortlinker/issues) 是否有类似问题
2. 提交新的 Issue，附上：
   - 错误信息截图
   - 浏览器控制台日志
   - 后端日志
   - 环境信息（操作系统、浏览器版本、Bun 版本）

## 相关链接

- 📖 [功能概述](./index)
- 🛠️ [开发指南](./development)
