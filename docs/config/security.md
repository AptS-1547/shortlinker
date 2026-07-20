# 安全最佳实践

本页聚焦与配置相关的安全建议，包括登录限流与 IPC Socket 权限控制。

### 登录限流配置

Shortlinker 仅在明确可信的代理连接上采信 `X-Forwarded-For`，该客户端 IP 同时用于登录限流和详细点击统计。

**直连部署**（无反向代理）：
- 无需额外配置，使用 TCP 连接的 peer IP，忽略客户端自行提交的 `X-Forwarded-For`

**反向代理部署**（Nginx/Caddy/Docker）：
- 在管理面板配置 `api.trusted_proxies`，列出 Shortlinker 直接连接到的代理 IP 或 CIDR
- 只有直接 peer 命中该列表时才采信 `X-Forwarded-For`；未命中时按代理 peer IP 限流和记录

**Unix Socket 连接**（nginx 同机器）：
- Shortlinker 自动信任本机 Unix socket 反代传输
- 确保 nginx 配置了 `proxy_set_header X-Forwarded-For $remote_addr;`
- 缺少该请求头时登录仍可用，但所有请求会退化为本机 loopback IP，共享同一个限流桶

示例配置（可选）：

保存 `api.trusted_proxies` 后需要重启 Shortlinker，确保登录限流器和详细点击统计使用同一份代理规则。

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
> - 不要因为代理位于内网就信任整段 VPC；优先填写 Shortlinker 直接连接到的代理地址
> - 错误配置可能导致所有用户共享同一限流桶（代理 IP 未匹配）或引入伪造风险（信任范围过宽）
> - 查看启动日志确认当前模式：`Client IP extraction: direct peer mode enabled` 或 `Client IP extraction: explicit trusted proxies configured`

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
