# Reverse Proxy Configuration

In production environments, it's recommended to expose Shortlinker service through a reverse proxy.

## Navigation

- [Performance and Monitoring](/en/deployment/proxy-operations)

::: warning Important: Reverse Proxy Configuration Requirements
When deploying behind a reverse proxy, it's recommended to set `X-Real-IP` and `X-Forwarded-For` so the server can extract the real client IP (used by login rate limiting and analytics).

- **TCP reverse proxy**: if missing, login usually still works, but rate limiting degrades to the proxy IP (all users share the same limiter key).
- **Unix socket mode** (when `server.unix_socket` is configured): `X-Forwarded-For` is **required**; otherwise the login rate limiter cannot extract a key and login will return HTTP 500.
:::

## Caddy Configuration

### Basic Configuration

```caddy
# TCP port
esap.cc {
    reverse_proxy 127.0.0.1:8080 {
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
    }

    # Optional: Add cache control
    header {
        Cache-Control "no-cache, no-store, must-revalidate"
    }
}

# Unix socket
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

### Configuration with SSL

```caddy
esap.cc {
    reverse_proxy 127.0.0.1:8080 {
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
    }

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

    # Forward client real IP (requires mod_headers)
    RequestHeader set X-Real-IP "%{REMOTE_ADDR}s"
    RequestHeader set X-Forwarded-For "%{REMOTE_ADDR}s"

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

    # Forward client real IP (requires mod_headers)
    RequestHeader set X-Real-IP "%{REMOTE_ADDR}s"
    RequestHeader set X-Forwarded-For "%{REMOTE_ADDR}s"

    Header always set Cache-Control "no-cache, no-store, must-revalidate"

    CustomLog /var/log/apache2/shortlinker.access.log combined
    ErrorLog /var/log/apache2/shortlinker.error.log
</VirtualHost>
```


## Next

- [Performance and Monitoring](/en/deployment/proxy-operations)
