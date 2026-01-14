# 反向代理配置

在生产环境中，建议通过反向代理来暴露 Shortlinker 服务。

## Caddy 配置

### 基础配置

```caddy
# TCP 端口
esap.cc {
    reverse_proxy 127.0.0.1:8080
    
    # 可选：添加缓存控制
    header {
        Cache-Control "no-cache, no-store, must-revalidate"
    }
}

# Unix 套接字
esap.cc {
    reverse_proxy unix//tmp/shortlinker.sock
    
    header {
        Cache-Control "no-cache, no-store, must-revalidate"
    }
}
```

### 带 SSL 的配置

```caddy
esap.cc {
    reverse_proxy 127.0.0.1:8080
    
    # 自动 HTTPS
    tls {
        protocols tls1.2 tls1.3
    }
    
    # 日志配置
    log {
        output file /var/log/caddy/shortlinker.log
        format single_field common_log
    }
}
```

## Nginx 配置

### 基础配置

```nginx
# TCP 端口
server {
    listen 80;
    server_name esap.cc;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # 禁用缓存
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}

# Unix 套接字
server {
    listen 80;
    server_name esap.cc;
    
    location / {
        proxy_pass http://unix:/tmp/shortlinker.sock;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}
```

### 完整 HTTPS 配置

```nginx
server {
    listen 80;
    server_name esap.cc;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name esap.cc;
    
    # SSL 配置
    ssl_certificate /etc/ssl/certs/esap.cc.crt;
    ssl_certificate_key /etc/ssl/private/esap.cc.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # 禁用缓存
        add_header Cache-Control "no-cache, no-store, must-revalidate";
        
        # 安全头
        add_header X-Frame-Options DENY;
        add_header X-Content-Type-Options nosniff;
        add_header X-XSS-Protection "1; mode=block";
    }
    
    # 日志配置
    access_log /var/log/nginx/shortlinker.access.log;
    error_log /var/log/nginx/shortlinker.error.log;
}
```

## Apache 配置

```apache
# TCP 端口
<VirtualHost *:80>
    ServerName esap.cc
    
    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:8080/
    ProxyPassReverse / http://127.0.0.1:8080/
    
    # 禁用缓存
    Header always set Cache-Control "no-cache, no-store, must-revalidate"
    
    # 日志
    CustomLog /var/log/apache2/shortlinker.access.log combined
    ErrorLog /var/log/apache2/shortlinker.error.log
</VirtualHost>

# Unix 套接字
<VirtualHost *:80>
    ServerName esap.cc
    
    ProxyPreserveHost On
    ProxyPass / unix:/tmp/shortlinker.sock|http://localhost/
    ProxyPassReverse / unix:/tmp/shortlinker.sock|http://localhost/
    
    Header always set Cache-Control "no-cache, no-store, must-revalidate"
    
    CustomLog /var/log/apache2/shortlinker.access.log combined
    ErrorLog /var/log/apache2/shortlinker.error.log
</VirtualHost>
```

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

> 注意：当前版本的 `/health/*` 端点需要 Admin 登录后下发的 JWT Cookie，不适合作为反向代理层的简单探活接口。
>
> 如果你只需要确认后端进程存活，建议探测根路径 `/`（默认会返回 `307`），或在反代层自定义一个探活路径转发到 `/`。

```nginx
location = /_healthz {
    access_log off;
    proxy_pass http://127.0.0.1:8080/;
    proxy_connect_timeout 1s;
    proxy_send_timeout 1s;
    proxy_read_timeout 1s;
}
```
