# 反向代理配置：性能优化与监控

本页聚焦负载均衡、性能优化和监控日志配置。

## 负载均衡

### Nginx 负载均衡

```nginx
upstream shortlinker {
    server 127.0.0.1:8080;
    server 127.0.0.1:8081;
    server 127.0.0.1:8082;
}

server {
    listen 80;
    server_name esap.cc;
    
    location / {
        proxy_pass http://shortlinker;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

## 性能优化

### 连接池优化

```nginx
upstream shortlinker {
    server 127.0.0.1:8080 max_fails=3 fail_timeout=30s;
    keepalive 32;
}

server {
    location / {
        proxy_pass http://shortlinker;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_connect_timeout 5s;
        proxy_send_timeout 5s;
        proxy_read_timeout 5s;
    }
}
```

### 缓存配置

虽然短链接不应该被缓存，但可以缓存静态资源：

```nginx
location ~* \.(jpg|jpeg|png|gif|ico|css|js)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}
```

## 监控和日志

### 访问日志格式

```nginx
log_format shortlinker '$remote_addr - $remote_user [$time_local] '
                      '"$request" $status $body_bytes_sent '
                      '"$http_referer" "$http_user_agent" '
                      '$request_time $upstream_response_time';

access_log /var/log/nginx/shortlinker.log shortlinker;
```

### 健康检查

> 注意：`/health/*` 端点默认需要鉴权。推荐在生产环境配置运行时配置 `api.health_token`，并使用 `Authorization: Bearer <token>` 探测 `/health/live` 或 `/health/ready`。  
> 如果不方便在探活请求中添加请求头，也可以探测根路径 `/`（默认返回 `307`）作为简单存活检查。

```nginx
location = /_healthz {
    access_log off;
    # 推荐：带 Bearer Token 的健康探测（需要你已配置 api.health_token）
    # proxy_set_header Authorization "Bearer your_health_token";
    # proxy_pass http://127.0.0.1:8080/health/live;

    # 兼容：不带鉴权的简单存活探测（探测根路径 /，返回 307 也视为存活）
    proxy_pass http://127.0.0.1:8080/;
    proxy_connect_timeout 1s;
    proxy_send_timeout 1s;
    proxy_read_timeout 1s;
}
```
