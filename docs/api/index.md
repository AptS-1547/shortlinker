# API 文档

Shortlinker 提供简洁的 HTTP API 接口用于短链接重定向。

## 接口概述

Shortlinker 主要提供一个重定向接口，支持 GET 和 HEAD 方法。

## 分区导航

- 重定向接口（本页）：`GET/HEAD /{path...}`
- [Admin API 概览](/api/admin)
- [Admin API：链接与批量操作](/api/admin-links)
- [Admin API：运行时配置与自动化示例](/api/admin-config)
- [Admin API：Analytics 统计分析](/api/admin-analytics)
- [健康检查 API 概览](/api/health)
- [健康检查 API：端点与状态码](/api/health-endpoints)
- [健康检查 API：监控集成与故障排除](/api/health-monitoring)

## 基础信息

- **Base URL**: `http://your-domain:port/`
- **协议**: HTTP/1.1
- **编码**: UTF-8
- **重定向类型**: 307 Temporary Redirect

## 接口详情

### GET/HEAD /{path...}

重定向到指定短码对应的目标 URL。

**请求方法**: `GET` | `HEAD`

**请求路径**: `/{path}`（支持多级路径，例如 `/foo/bar`）

**路径参数**:
- `path` (string): 短链接代码（大小写敏感）

**短码格式约束**（不满足会直接返回 `404`）：
- 最大长度：128
- 允许字符：`[a-zA-Z0-9_.-/]`

> 注意：`routes.admin_prefix` / `routes.health_prefix` / `routes.frontend_prefix` 对应的路径前缀是保留路由（默认 `/admin` / `/health` / `/panel`），不会命中重定向接口；短链接 `code` 也不能与这些前缀冲突（如 `admin` 或 `admin/...`），否则创建会被拒绝。

**响应**:

#### 成功重定向 (307)
```http
HTTP/1.1 307 Temporary Redirect
Location: https://example.com
Cache-Control: no-cache, no-store, must-revalidate
```

> **UTM 透传（可选）**：
> - 当运行时配置 `utm.enable_passthrough=true` 时，会把请求中的以下参数透传到目标 URL：`utm_source`、`utm_medium`、`utm_campaign`、`utm_term`、`utm_content`。
> - 仅透传这 5 个键，其它 Query 参数不会追加到目标 URL。

示例：

```http
GET /promo?utm_source=newsletter&utm_campaign=spring HTTP/1.1

HTTP/1.1 307 Temporary Redirect
Location: https://example.com/landing?utm_source=newsletter&utm_campaign=spring
```

#### 短码不存在/已过期 (404)
```http
HTTP/1.1 404 Not Found
Content-Type: text/html; charset=utf-8
Cache-Control: public, max-age=60

Not Found
```

> **注意**：404 响应使用 `Cache-Control: public, max-age=60` 进行短时缓存，以减少对不存在短码的重复请求。

#### 服务内部错误 (500)
```http
HTTP/1.1 500 Internal Server Error
Content-Type: text/html; charset=utf-8

Internal Server Error
```

> 通常表示存储层查询异常（例如数据库暂时不可用）；可结合服务日志中的 `error` 级别信息排查。

## 特殊路径

### 根路径重定向

当访问根路径 `/` 时，会重定向到默认 URL（运行时配置项 `features.default_url`）。

**请求**:
```http
GET / HTTP/1.1
Host: localhost:8080
```

**响应**:
```http
HTTP/1.1 307 Temporary Redirect
Location: https://esap.cc/repo
```

## 使用示例

### curl 示例

```bash
# 重定向请求
curl -I http://localhost:8080/example
# HTTP/1.1 307 Temporary Redirect
# Location: https://www.example.com

# 跟随重定向
curl -L http://localhost:8080/example

# 不存在的短码
curl -I http://localhost:8080/nonexistent
# HTTP/1.1 404 Not Found
```

### JavaScript 示例

```javascript
async function checkShortLink(shortCode) {
    try {
        const response = await fetch(`http://localhost:8080/${shortCode}`, {
            method: 'HEAD',
            redirect: 'manual'
        });
        
        if (response.status === 307) {
            return response.headers.get('Location');
        } else {
            return null;
        }
    } catch (error) {
        console.error('检查失败:', error);
        return null;
    }
}
```

### Python 示例

```python
import requests

def check_short_link(base_url, short_code):
    """检查短链接并返回目标 URL"""
    try:
        response = requests.head(
            f"{base_url}/{short_code}",
            allow_redirects=False
        )
        return response.headers.get('Location') if response.status_code == 307 else None
    except requests.RequestException:
        return None
```

## 缓存策略

重定向响应（307）包含 `Cache-Control: no-cache, no-store, must-revalidate` 头，确保：
- 浏览器不会缓存重定向响应
- 短链接修改能立即生效
- 过期检查实时进行

404 响应使用 `Cache-Control: public, max-age=60`，允许短时缓存以减少无效请求。

## 性能特征

- **响应时间**: < 1ms（SQLite 本地存储）
- **并发支持**: 数千个并发连接
- **内存使用**: 极低内存占用
- **存储后端**: 支持 SQLite、MySQL、PostgreSQL、MariaDB

## 监控和日志

当前实现的重定向链路默认仅记录必要日志（是否输出取决于日志级别）：
- `trace`：非法短码拒绝等细粒度行为
- `debug`：缓存未命中、短码不存在等非错误分支
- `error`：数据库查询异常（对应返回 `500`）

如果启用 `metrics` feature，可通过 `shortlinker_redirects_total{status="307"|"404"|"500"}` 观测重定向状态分布。

## UTM 来源解析（详细日志）

当 `analytics.enable_detailed_logging=true` 时，每次点击会写入 `click_logs.source`，来源推导规则如下：

1. 请求 URL 存在 `utm_source`：直接使用该值；
2. 否则若请求头有 `Referer`：记录为 `ref:{domain}`（例如 `ref:google.com`）；
3. 否则记录为 `direct`。
