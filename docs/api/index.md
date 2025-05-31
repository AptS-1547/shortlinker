# API 文档

Shortlinker 提供简洁的 HTTP API 接口用于短链接重定向。

## 接口概述

Shortlinker 主要提供一个重定向接口，支持 GET 和 HEAD 方法。

## 基础信息

- **Base URL**: `http://your-domain:port/`
- **协议**: HTTP/1.1
- **编码**: UTF-8
- **重定向类型**: 307 Temporary Redirect

## 接口详情

### GET/HEAD /{path}

重定向到指定短码对应的目标 URL。

**请求方法**: `GET` | `HEAD`

**请求路径**: `/{short_code}`

**路径参数**:
- `short_code` (string): 短链接代码

**响应**:

#### 成功重定向 (307)
```http
HTTP/1.1 307 Temporary Redirect
Location: https://example.com
Cache-Control: no-cache, no-store, must-revalidate
```

#### 短码不存在/已过期 (404)
```http
HTTP/1.1 404 Not Found
Content-Type: text/html; charset=utf-8
Cache-Control: no-cache, no-store, must-revalidate

Not Found
```

## 特殊路径

### 根路径重定向

当访问根路径 `/` 时，会重定向到默认 URL（通过 `DEFAULT_URL` 环境变量配置）。

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

所有响应都包含 `Cache-Control: no-cache, no-store, must-revalidate` 头，确保：
- 浏览器不会缓存重定向响应
- 短链接修改能立即生效
- 过期检查实时进行

## 性能特征

- **响应时间**: < 1ms（SQLite/文件存储）
- **并发支持**: 数千个并发连接
- **内存使用**: 极低内存占用
- **存储后端**: 支持 SQLite、文件、Sled 多种存储

## 监控和日志

服务器会记录以下信息：
- 重定向操作日志
- 404 错误日志
- 过期链接访问日志

日志示例：
```
[INFO] 重定向 example -> https://www.example.com
[INFO] 链接已过期: temp
```
