# Reverse Proxy Configuration

In production environments, it's recommended to expose Shortlinker service through a reverse proxy.

## Caddy Configuration

### Basic Configuration

```caddy
# TCP port
esap.cc {
    reverse_proxy 127.0.0.1:8080
    
    # Optional: Add cache control
    header {
        Cache-Control "no-cache, no-store, must-revalidate"
    }
}

# Unix socket
esap.cc {
    reverse_proxy unix//tmp/shortlinker.sock
    
    header {
        Cache-Control "no-cache, no-store, must-revalidate"
    }
}
```

### Configuration with SSL

```caddy
esap.cc {
    reverse_proxy 127.0.0.1:8080
    
    # Automatic HTTPS
    tls {
        protocols tls1.2 tls1.3
    }
    
    # Log configuration
    log {
        output file /var/log/caddy/shortlinker.log
        format single_field common_log
    }
}
```

## Nginx Configuration

### Basic Configuration

```nginx
# TCP port
server {
    listen 80;
    server_name esap.cc;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # Disable cache
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}

# Unix socket
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

### Complete HTTPS Configuration

```nginx
server {
    listen 80;
    server_name esap.cc;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name esap.cc;
    
    # SSL configuration
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
        
        # Disable cache
        add_header Cache-Control "no-cache, no-store, must-revalidate";
        
        # Security headers
        add_header X-Frame-Options DENY;
        add_header X-Content-Type-Options nosniff;
        add_header X-XSS-Protection "1; mode=block";
    }
    
    # Log configuration
    access_log /var/log/nginx/shortlinker.access.log;
    error_log /var/log/nginx/shortlinker.error.log;
}
```

## Apache Configuration

```apache
# TCP port
<VirtualHost *:80>
    ServerName esap.cc
    
    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:8080/
    ProxyPassReverse / http://127.0.0.1:8080/
    
    # Disable cache
    Header always set Cache-Control "no-cache, no-store, must-revalidate"
    
    # Logs
    CustomLog /var/log/apache2/shortlinker.access.log combined
    ErrorLog /var/log/apache2/shortlinker.error.log
</VirtualHost>

# Unix socket
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

> Note: In the current version, `/health/*` endpoints require JWT cookies issued after Admin login, so they are not suitable as a simple reverse-proxy health probe.
>
> If you only need a liveness probe, check `/` (default returns `307`), or expose a dedicated proxy health path that forwards to `/`.

```nginx
location = /_healthz {
    access_log off;
    proxy_pass http://127.0.0.1:8080/;
    proxy_connect_timeout 1s;
    proxy_send_timeout 1s;
    proxy_read_timeout 1s;
}
```
