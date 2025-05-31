# Reverse Proxy Configuration

In production environments, it's recommended to expose Shortlinker service through a reverse proxy.

## Caddy Configuration

### Basic Configuration

```caddy
your-domain.com {
    reverse_proxy 127.0.0.1:8080
    
    # Optional: Add cache control
    header {
        Cache-Control "no-cache, no-store, must-revalidate"
    }
}
```

### SSL Configuration

```caddy
your-domain.com {
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

### Admin API Protection

```caddy
your-domain.com {
    # Main shortlinker service
    reverse_proxy 127.0.0.1:8080
    
    # Protect Admin API with IP restriction
    @admin {
        path /admin/*
        not remote_ip 192.168.1.0/24 10.0.0.0/8
    }
    respond @admin "Access Denied" 403
    
    # Add security headers
    header {
        Cache-Control "no-cache, no-store, must-revalidate"
        X-Frame-Options "DENY"
        X-Content-Type-Options "nosniff"
    }
}
```

## Nginx Configuration

### Basic Configuration

```nginx
server {
    listen 80;
    server_name your-domain.com;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # Disable caching
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}
```

### Complete HTTPS Configuration

```nginx
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;
    
    # SSL configuration
    ssl_certificate /etc/ssl/certs/your-domain.com.crt;
    ssl_certificate_key /etc/ssl/private/your-domain.com.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Disable caching
        add_header Cache-Control "no-cache, no-store, must-revalidate";
        
        # Security headers
        add_header X-Frame-Options DENY;
        add_header X-Content-Type-Options nosniff;
        add_header X-XSS-Protection "1; mode=block";
        add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    }
    
    # Admin API protection
    location /admin {
        allow 192.168.1.0/24;
        allow 10.0.0.0/8;
        deny all;
        
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # Logging
    access_log /var/log/nginx/shortlinker.access.log;
    error_log /var/log/nginx/shortlinker.error.log;
}
```

## Apache Configuration

### Basic Configuration

```apache
<VirtualHost *:80>
    ServerName your-domain.com
    
    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:8080/
    ProxyPassReverse / http://127.0.0.1:8080/
    
    # Disable caching
    Header always set Cache-Control "no-cache, no-store, must-revalidate"
    
    # Logging
    CustomLog /var/log/apache2/shortlinker.access.log combined
    ErrorLog /var/log/apache2/shortlinker.error.log
</VirtualHost>
```

### HTTPS with Admin API Protection

```apache
<VirtualHost *:443>
    ServerName your-domain.com
    
    SSLEngine on
    SSLCertificateFile /etc/ssl/certs/your-domain.com.crt
    SSLCertificateKeyFile /etc/ssl/private/your-domain.com.key
    
    # Main proxy
    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:8080/
    ProxyPassReverse / http://127.0.0.1:8080/
    
    # Admin API protection
    <Location "/admin">
        Require ip 192.168.1
        Require ip 10.0.0
    </Location>
    
    # Security headers
    Header always set Cache-Control "no-cache, no-store, must-revalidate"
    Header always set X-Frame-Options "DENY"
    Header always set X-Content-Type-Options "nosniff"
    Header always set Strict-Transport-Security "max-age=31536000; includeSubDomains"
</VirtualHost>
```

## Load Balancing

### Nginx Load Balancing

```nginx
upstream shortlinker {
    server 127.0.0.1:8080 weight=3 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:8081 weight=2 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:8082 weight=1 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    server_name your-domain.com;
    
    location / {
        proxy_pass http://shortlinker;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # Load balancing settings
        proxy_next_upstream error timeout invalid_header http_500 http_502 http_503 http_504;
        proxy_connect_timeout 5s;
        proxy_send_timeout 5s;
        proxy_read_timeout 5s;
    }
}
```

## Performance Optimization

### Connection Pool Optimization

```nginx
upstream shortlinker {
    server 127.0.0.1:8080 max_fails=3 fail_timeout=30s;
    keepalive 32;
    keepalive_requests 100;
    keepalive_timeout 60s;
}

server {
    location / {
        proxy_pass http://shortlinker;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_connect_timeout 5s;
        proxy_send_timeout 5s;
        proxy_read_timeout 5s;
        
        # Buffer settings
        proxy_buffering on;
        proxy_buffer_size 4k;
        proxy_buffers 8 4k;
    }
}
```

### Rate Limiting

```nginx
# Define rate limiting zones
limit_req_zone $binary_remote_addr zone=general:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=admin:10m rate=1r/s;

server {
    # General rate limiting
    location / {
        limit_req zone=general burst=20 nodelay;
        proxy_pass http://127.0.0.1:8080;
    }
    
    # Stricter rate limiting for Admin API
    location /admin {
        limit_req zone=admin burst=5 nodelay;
        proxy_pass http://127.0.0.1:8080;
    }
}
```

## Monitoring and Logging

### Access Log Format

```nginx
log_format shortlinker '$remote_addr - $remote_user [$time_local] '
                      '"$request" $status $body_bytes_sent '
                      '"$http_referer" "$http_user_agent" '
                      '$request_time $upstream_response_time '
                      '$upstream_addr $upstream_status';

access_log /var/log/nginx/shortlinker.log shortlinker;
```

### Health Check

```nginx
location /health {
    access_log off;
    proxy_pass http://127.0.0.1:8080/;
    proxy_connect_timeout 1s;
    proxy_send_timeout 1s;
    proxy_read_timeout 1s;
    
    # Return 503 if backend is down
    error_page 502 503 504 =503 /503.html;
}

location = /503.html {
    root /var/www/html;
    internal;
}
```

### Real-time Monitoring

```bash
# Monitor access logs
tail -f /var/log/nginx/shortlinker.log | grep -E "(admin|error)"

# Monitor response times
tail -f /var/log/nginx/shortlinker.log | awk '{print $NF}' | grep -v '-'
```

## Security Best Practices

### IP Whitelisting for Admin API

```nginx
# Create allowed IPs file
echo "192.168.1.0/24" > /etc/nginx/admin_ips.conf
echo "10.0.0.0/8" >> /etc/nginx/admin_ips.conf

# Use in configuration
location /admin {
    include /etc/nginx/admin_ips.conf;
    deny all;
    proxy_pass http://127.0.0.1:8080;
}
```

### DDoS Protection

```nginx
# Connection limiting
limit_conn_zone $binary_remote_addr zone=conn_limit_per_ip:10m;
limit_conn conn_limit_per_ip 20;

# Request size limiting
client_max_body_size 1m;
large_client_header_buffers 2 1k;
```

### SSL Security

```nginx
# Modern SSL configuration
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
ssl_prefer_server_ciphers off;

# HSTS
add_header Strict-Transport-Security "max-age=63072000" always;

# OCSP Stapling
ssl_stapling on;
ssl_stapling_verify on;
```

## File Permissions

```bash
# Set correct permissions
sudo chown -R www-data:www-data /opt/shortlinker
sudo chmod 755 /opt/shortlinker
sudo chmod 600 /opt/shortlinker/data/links.db  # SQLite database
sudo chmod 755 /opt/shortlinker/shortlinker

# Secure Admin API token in environment
sudo chmod 600 /opt/shortlinker/.env
```
