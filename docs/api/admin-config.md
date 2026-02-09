# Admin API：运行时配置与自动化示例

本页聚焦运行时配置管理接口、认证接口补充说明与 Python 自动化调用示例。

## 运行时配置管理

配置管理接口位于 `/{ADMIN_ROUTE_PREFIX}/v1/config` 下，返回值统一为 `{code,message,data}` 结构；敏感配置会自动掩码为 `[REDACTED]`。

### GET /config - 获取所有配置

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/config
```

### GET /config/schema - 获取配置 Schema（元信息）

返回所有配置项的元信息（类型、默认值、是否需要重启、枚举选项等），主要用于前端动态渲染配置表单/校验。

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/config/schema
```

### GET /config/{key} - 获取单个配置

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/config/features.random_code_length
```

### PUT /config/{key} - 更新配置

```bash
curl -sS -X PUT \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"value":"8"}' \
  http://localhost:8080/admin/v1/config/features.random_code_length
```

### GET /config/{key}/history - 获取变更历史

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/config/features.random_code_length/history?limit=10"
```

> `limit` 默认 `20`，服务端会将最大值限制为 `100`。


### POST /config/reload - 重新加载配置

```bash
curl -sS -X POST -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  http://localhost:8080/admin/v1/config/reload
```

### POST /config/{key}/action - 执行配置 Action（不保存）

执行配置项支持的 Action，并返回执行结果，但**不会**写回数据库。

> 注意：该接口会在响应中返回 `data.value`（可能是敏感值，例如生成的 token）。如需“只保存不回显”，请使用 `/execute-and-save`。


```bash
curl -sS -X POST \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"action":"generate_token"}' \
  http://localhost:8080/admin/v1/config/api.jwt_secret/action
```

### POST /config/{key}/execute-and-save - 执行并保存 Action

执行 Action 并将结果直接保存到配置中（响应不会返回敏感值本体）。

```bash
curl -sS -X POST \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"action":"generate_token"}' \
  http://localhost:8080/admin/v1/config/api.jwt_secret/execute-and-save
```

> 当前仅 `api.jwt_secret` 支持 `generate_token` Action。

## 认证接口补充说明

- `POST /auth/login`：无需 Cookie；验证管理员登录密码（与 `api.admin_token` 的 Argon2 哈希匹配）成功后下发 Cookie
- `POST /auth/refresh`：无需 Access Cookie，但需要 Refresh Cookie
- `POST /auth/logout`：无需 Cookie；用于清理 Cookie
- `GET /auth/verify`：需要有效 Access 凭证（Access Cookie 或 Bearer Access Token）

## Python 客户端示例（requests）

```python
import requests

class ShortlinkerAdmin:
    def __init__(self, base_url: str, admin_token: str):
        self.base_url = base_url.rstrip("/")
        self.session = requests.Session()
        self.csrf_token = None

        # 登录：Set-Cookie 会被 requests.Session 自动保存
        resp = self.session.post(
            f"{self.base_url}/admin/v1/auth/login",
            json={"password": admin_token},
            timeout=10,
        )
        resp.raise_for_status()
        self.csrf_token = self.session.cookies.get("csrf_token")

    def get_all_links(self, page=1, page_size=20):
        resp = self.session.get(
            f"{self.base_url}/admin/v1/links",
            params={"page": page, "page_size": page_size},
            timeout=10,
        )
        resp.raise_for_status()
        return resp.json()

    def create_link(self, code, target, expires_at=None, force=False):
        payload = {"code": code, "target": target, "force": force}
        if expires_at:
            payload["expires_at"] = expires_at
        resp = self.session.post(
            f"{self.base_url}/admin/v1/links",
            headers={"X-CSRF-Token": self.csrf_token or ""},
            json=payload,
            timeout=10,
        )
        resp.raise_for_status()
        return resp.json()

# 使用示例
admin = ShortlinkerAdmin("http://localhost:8080", "your_admin_token")
print(admin.get_all_links())
```
