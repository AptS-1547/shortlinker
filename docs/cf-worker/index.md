# Cloudflare Worker 版本（实验中）

`cf-worker` 目录中的 Cloudflare Worker 方案目前仍处于开发阶段，用于探索无服务器部署路径。

## 当前状态

- 🚧 **开发中**：尚未达到生产可用状态
- 📦 **代码位置**：仓库根目录 `cf-worker/`
- 📖 **最新进度**：以 `cf-worker/README.zh.md`（中文）和 `cf-worker/README.md`（英文）为准

## 已有内容

- Worker 工程骨架与基础构建配置
- Rust + WebAssembly + Wrangler 的开发链路
- 本地开发与试运行命令

## 尚未完成（以 README 路线图为准）

- 完整短链创建/查询/管理能力
- 完整 Admin API 与鉴权流程
- 分析统计、限流与生产级防护能力

## 本地试运行（开发用途）

```bash
cd cf-worker
cargo build
wrangler dev
```

## 部署说明

- 当前文档不再将 Worker 视为“可直接上线”的生产方案。
- 当 README 明确标记可部署后，使用 `wrangler deploy` 进行发布。
