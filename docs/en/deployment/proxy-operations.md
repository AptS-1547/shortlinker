# Reverse Proxy: Performance and Monitoring

This page focuses on load balancing, performance tuning, and monitoring/logging setup.

## Load Balancing

### Nginx Load Balancing

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

## Performance Optimization

### Connection Pool Optimization

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

### Cache Configuration

Although short links shouldn't be cached, static resources can be cached:

```nginx
location ~* \.(jpg|jpeg|png|gif|ico|css|js)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}
```

## Monitoring and Logging

### Access Log Format

```nginx
log_format shortlinker '$remote_addr - $remote_user [$time_local] '
                      '"$request" $status $body_bytes_sent '
                      '"$http_referer" "$http_user_agent" '
                      '$request_time $upstream_response_time';

access_log /var/log/nginx/shortlinker.log shortlinker;
```

### Health Check

> Note: `/health/*` endpoints require authentication by default. In production, it’s recommended to set runtime config `api.health_token` and probe `/health/live` or `/health/ready` with `Authorization: Bearer <token>`.  
> If adding request headers is not convenient, probing `/` (default returns `307`) can be used as a simple liveness check.

```nginx
location = /_healthz {
    access_log off;
    # Recommended: authenticated probe (requires api.health_token configured)
    # proxy_set_header Authorization "Bearer your_health_token";
    # proxy_pass http://127.0.0.1:8080/health/live;

    # Fallback: simple liveness probe (probes `/`, 307 counts as alive)
    proxy_pass http://127.0.0.1:8080/;
    proxy_connect_timeout 1s;
    proxy_send_timeout 1s;
    proxy_read_timeout 1s;
}
```
