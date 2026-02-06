# Admin API：链接与批量操作

本页聚焦短链接 CRUD、批量操作与 CSV 导入/导出。鉴权方式与通用响应格式见 [Admin API 概览](/api/admin)。

## 链接管理

### GET /links - 获取短链接列表（分页 + 过滤）

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/links?page=1&page_size=20"
```

**查询参数**：

| 参数 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `page` | Integer | 页码（从 1 开始） | `?page=1` |
| `page_size` | Integer | 每页数量（1-100） | `?page_size=20` |
| `search` | String | 模糊搜索短码和目标 URL | `?search=github` |
| `created_after` | RFC3339 | 创建时间过滤（晚于） | `?created_after=2024-01-01T00:00:00Z` |
| `created_before` | RFC3339 | 创建时间过滤（早于） | `?created_before=2024-12-31T23:59:59Z` |
| `only_expired` | Boolean | 仅显示已过期 | `?only_expired=true` |
| `only_active` | Boolean | 仅显示未过期 | `?only_active=true` |

**响应格式**（分页）：
```json
{
  "code": 0,
  "message": "OK",
  "data": [
    {
      "code": "github",
      "target": "https://github.com",
      "created_at": "2024-12-15T14:30:22Z",
      "expires_at": null,
      "password": null,
      "click_count": 42
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total": 42,
    "total_pages": 3
  }
}
```

### POST /links - 创建短链接

```bash
curl -sS -X POST \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"code":"github","target":"https://github.com"}' \
  http://localhost:8080/admin/v1/links
```

**请求体**：
```json
{
  "code": "github",
  "target": "https://github.com",
  "expires_at": "2024-12-31T23:59:59Z",
  "password": "secret123",
  "force": false
}
```

**说明**：
- `code`：短码（可选），不提供则自动生成随机短码
  - 格式约束：非空、长度 ≤ 128，字符集 `[a-zA-Z0-9_.-/]`（支持多级路径）
  - 不能与保留路由前缀冲突：默认 `admin` / `health` / `panel`（来自 `routes.*_prefix`），即短码不能等于这些前缀，也不能以 `{prefix}/` 开头
- `target`：目标 URL（必需）
- `expires_at`：过期时间（可选），支持相对时间（如 `"1d"`, `"7d"`, `"1w"`）或 RFC3339
- `force`：当 `code` 已存在时，是否覆盖（可选，默认 `false`；未开启时会返回 `409 Conflict`）
- `password`：密码保护字段（实验性）
  - 通过 Admin API 写入时会自动使用 Argon2 哈希（若传入的字符串已是 `$argon2...` 格式则会原样保存）
  - 当前版本重定向时不验证密码，仅存储

### GET /links/{code} - 获取指定短链接

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/links/github
```

### PUT /links/{code} - 更新短链接

```bash
curl -sS -X PUT \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"target":"https://github.com/new-repo","expires_at":"30d"}' \
  http://localhost:8080/admin/v1/links/github
```

**请求体说明**：
```json
{
  "target": "https://new-url.com",
  "expires_at": "7d",
  "password": ""
}
```

**说明**：
- `target` 必填
- `expires_at` 不提供则保持原值
- `password`
  - 不提供：保持原值
  - 传空字符串 `""`：清除密码
  - 传明文：自动 Argon2 哈希后保存
  - 传 `$argon2...`：视为已哈希，原样保存

### DELETE /links/{code} - 删除短链接

```bash
curl -sS -X DELETE -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  http://localhost:8080/admin/v1/links/github
```

### GET /stats - 获取统计信息

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/stats
```

**响应格式**：
```json
{
  "code": 0,
  "message": "OK",
  "data": {
    "total_links": 100,
    "total_clicks": 5000,
    "active_links": 80
  }
}
```

## 批量操作

### POST /links/batch - 批量创建短链接

> 注意：请求体是对象，字段名为 `links`，不是纯数组。
>
> `links[].code` 同样适用上文的短码格式/保留前缀约束。

```bash
curl -sS -X POST \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"links":[{"code":"link1","target":"https://example1.com"},{"code":"link2","target":"https://example2.com"}]}' \
  http://localhost:8080/admin/v1/links/batch
```

### PUT /links/batch - 批量更新短链接

> 注意：请求体字段名为 `updates`，每一项包含 `code` 与 `payload`。

```bash
curl -sS -X PUT \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"updates":[{"code":"link1","payload":{"target":"https://new-example1.com"}},{"code":"link2","payload":{"target":"https://new-example2.com"}}]}' \
  http://localhost:8080/admin/v1/links/batch
```

### DELETE /links/batch - 批量删除短链接

> 注意：请求体字段名为 `codes`。

```bash
curl -sS -X DELETE \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"codes":["link1","link2","link3"]}' \
  http://localhost:8080/admin/v1/links/batch
```

## CSV 导出/导入

### GET /links/export - 导出为 CSV

导出会生成可直接用于导入的 CSV（包含 header），字段：
`code,target,created_at,expires_at,password,click_count`

```bash
curl -sS -b cookies.txt \
  -o shortlinks_export.csv \
  "http://localhost:8080/admin/v1/links/export?only_active=true"
```

### POST /links/import - 从 CSV 导入

上传 `multipart/form-data`：
- `file`：CSV 文件
- `mode`（可选）：冲突处理模式，`skip`（默认）/`overwrite`/`error`

```bash
curl -sS -X POST \
  -b cookies.txt -c cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -F "mode=overwrite" \
  -F "file=@./shortlinks_export.csv" \
  http://localhost:8080/admin/v1/links/import
```

**响应**：
```json
{
  "code": 0,
  "message": "OK",
  "data": {
    "total_rows": 10,
    "success_count": 9,
    "skipped_count": 1,
    "failed_count": 0,
    "failed_items": []
  }
}
```
