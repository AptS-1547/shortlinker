# 反向代理配置

在生产环境中，建议通过反向代理来暴露 Shortlinker 服务。

## 文档导航

- [性能优化与监控](/deployment/proxy-operations)

::: warning 重要：反向代理配置要求
通过反向代理部署时，建议设置 `X-Real-IP` 和 `X-Forwarded-For` 请求头，用于获取客户端真实 IP（登录限流/统计等功能会用到）。

- **TCP 反代**：如果未设置这些头，登录通常仍可用，但登录限流会退化为按“代理 IP”限流（所有用户共享同一个限流 key）。
- **Unix Socket 模式**（配置了 `server.unix_socket`）：**必须**设置 `X-Forwarded-For`，否则登录限流无法提取 key，登录会返回 500。
:::

## Caddy 配置

### 基础配置

```caddy
# TCP 端口
esap.cc {
    reverse_proxy 127.0.0.1:8080 {
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
    }

    # 可选：添加缓存控制
    header {
        Cache-Control "no-cache, no-store, must-revalidate"
    }
}

# Unix 套接字
esap.cc {
    reverse_proxy unix//tmp/shortlinker.sock {
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
    }

    header {
        Cache-Control "no-cache, no-store, must-revalidate"
    }
}
```

### 带 SSL 的配置

```caddy
esap.cc {
    reverse_proxy 127.0.0.1:8080 {
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
    }

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

    # 传递客户端真实 IP（需要 mod_headers）
    RequestHeader set X-Real-IP "%{REMOTE_ADDR}s"
    RequestHeader set X-Forwarded-For "%{REMOTE_ADDR}s"

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

    # 传递客户端真实 IP（需要 mod_remoteip）
    RequestHeader set X-Real-IP "%{REMOTE_ADDR}s"
    RequestHeader set X-Forwarded-For "%{REMOTE_ADDR}s"

    Header always set Cache-Control "no-cache, no-store, must-revalidate"

    CustomLog /var/log/apache2/shortlinker.access.log combined
    ErrorLog /var/log/apache2/shortlinker.error.log
</VirtualHost>
```


## 下一步

- [性能优化与监控](/deployment/proxy-operations)
