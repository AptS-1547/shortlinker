# Cloudflare Worker 版本

项目附带了基于 Cloudflare Workers 的无服务器版本，位于 `cf-worker` 目录，适合部署到 Cloudflare 边缘网络。

## 特性

- ☁️ 全局分布，访问速度快
- 🗄️ 使用 KV 存储持久化短链接
- 💸 按需计费，免运维

## 部署步骤

1. 安装 [Wrangler](https://developers.cloudflare.com/workers/wrangler/)
2. 配置 `wrangler.toml` 中的账户信息和 KV 命名空间
3. 执行 `wrangler publish` 部署

示例：
```bash
cd cf-worker
wrangler publish
```

更多细节请查看 [`cf-worker/README.md`](../../cf-worker/README.md)。
