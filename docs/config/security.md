# 安全最佳实践

本页聚焦与配置相关的安全建议，包括登录限流与 IPC Socket 权限控制。

### 登录限流配置

Shortlinker 使用智能代理检测进行登录限流 IP 提取，兼顾安全性和易用性。

**直连部署**（无反向代理）：
- 无需额外配置，公网 IP 不会信任 `X-Forwarded-For`，安全且自动

**反向代理部署**（Nginx/Caddy/Docker）：
- **自动检测**（推荐）：无需配置 `api.trusted_proxies`，连接来自私有 IP（10.x、172.16-31.x、192.168.x）或 localhost 时自动信任 `X-Forwarded-For`
- **显式配置**：如需精确控制，可在管理面板配置 `api.trusted_proxies`，列出可信代理的 IP 或 CIDR

**Unix Socket 连接**（nginx 同机器）：
- 自动使用 `X-Forwarded-For` 提取客户端真实 IP
- 确保 nginx 配置了 `proxy_set_header X-Forwarded-For $remote_addr;`

示例配置（可选）：

```bash
CSRF_TOKEN=$(awk '$6=="csrf_token"{print $7}' cookies.txt | tail -n 1)

# Nginx 在本地
curl -X PUT -b cookies.txt \
     -H "X-CSRF-Token: ${CSRF_TOKEN}" \
     -H "Content-Type: application/json" \
     -d '{"value": "[\"127.0.0.1\"]"}' \
     http://localhost:8080/admin/v1/config/api.trusted_proxies

# Cloudflare CDN（使用 Cloudflare IP 段）
curl -X PUT -b cookies.txt \
     -H "X-CSRF-Token: ${CSRF_TOKEN}" \
     -H "Content-Type: application/json" \
     -d '{"value": "[\"103.21.244.0/22\", \"103.22.200.0/22\"]"}' \
     http://localhost:8080/admin/v1/config/api.trusted_proxies
```

> **注意**：
> - **智能检测模式**（默认）：适合绝大多数场景，但如果 shortlinker 直接绑定在 VPC 内网 IP 且无代理，建议显式配置 `trusted_proxies` 防止伪造攻击
> - **显式配置模式**：错误配置可能导致所有用户共享同一限流桶（代理 IP 未匹配）或重新引入绕过风险（信任了不安全的代理）
> - 查看启动日志确认当前检测模式：`Login rate limiting: Auto-detect mode enabled` 或 `Explicit trusted proxies configured`

### IPC Socket 权限

Unix 下 IPC socket 文件（默认为 `./shortlinker.sock`，也可能是你配置的 `ipc.socket_path` 或 CLI `--socket` 覆盖路径）权限会被自动设置为 `0600`（仅属主可访问），防止本地其他用户绕过 Admin API。

如果需要允许特定用户访问 CLI：
```bash
# 方法 1: 使用 setfacl 添加访问权限
setfacl -m u:username:rw ./shortlinker.sock

# 方法 2: 使用用户组
chgrp shortlinker-users ./shortlinker.sock
chmod 660 ./shortlinker.sock
```
