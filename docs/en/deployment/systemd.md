# System Service Configuration

Configure Shortlinker as a system service for auto-start and service management.

## systemd Service

### Create Service File

Create `/etc/systemd/system/shortlinker.service`:

```ini
[Unit]
Description=Shortlinker URL Shortening Service
After=network.target
Wants=network.target

[Service]
Type=simple
User=www-data
Group=www-data
WorkingDirectory=/opt/shortlinker
ExecStart=/opt/shortlinker/shortlinker
Restart=always
RestartSec=5
KillMode=mixed
KillSignal=SIGTERM
TimeoutStopSec=5

# Environment variables - TCP port
Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080

# Environment variables - Unix socket (choose one)
# Environment=UNIX_SOCKET=/tmp/shortlinker.sock

Environment=DATABASE_URL=sqlite:///opt/shortlinker/data/links.db
Environment=DEFAULT_URL=https://example.com
Environment=RUST_LOG=info

# Security configuration
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/shortlinker/data

[Install]
WantedBy=multi-user.target
```

### Deployment Steps

```bash
# Create user and directories
sudo useradd --system --shell /bin/false --home /opt/shortlinker www-data
sudo mkdir -p /opt/shortlinker/{data,logs}
sudo chown -R www-data:www-data /opt/shortlinker

# Copy binary file
sudo cp shortlinker /opt/shortlinker/
sudo chmod +x /opt/shortlinker/shortlinker

# Install and start service
sudo systemctl daemon-reload
sudo systemctl enable shortlinker
sudo systemctl start shortlinker
```

### Service Management

```bash
# Check status
sudo systemctl status shortlinker

# Start/stop/restart service
sudo systemctl start shortlinker
sudo systemctl stop shortlinker
sudo systemctl restart shortlinker

# View logs
sudo journalctl -u shortlinker -f
sudo journalctl -u shortlinker --since "1 hour ago"
```

## Docker Compose Service

### Production Environment Configuration

```yaml
version: '3.8'

services:
  shortlinker:
    image: e1saps/shortlinker:latest
    container_name: shortlinker-prod
    restart: unless-stopped
    ports:
      - "127.0.0.1:8080:8080"
    volumes:
      - ./data:/data
    environment:
      - SERVER_HOST=0.0.0.0
      - DATABASE_URL=sqlite:///data/shortlinker.db
      - DEFAULT_URL=https://your-domain.com
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "wget", "--quiet", "--tries=1", "--spider", "http://localhost:8080/"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    deploy:
      resources:
        limits:
          memory: 128M
          cpus: '0.5'
```

### Service Management

```bash
# Start service
docker-compose up -d

# Check status and logs
docker-compose ps
docker-compose logs -f shortlinker

# Stop and restart
docker-compose down
docker-compose restart shortlinker
```

## Monitoring and Logging

### Service Monitoring Script

```bash
#!/bin/bash
# monitor.sh - Service monitoring script

SERVICE_NAME="shortlinker"
LOG_FILE="/var/log/shortlinker-monitor.log"

check_service() {
    if systemctl is-active --quiet $SERVICE_NAME; then
        echo "$(date): $SERVICE_NAME is running" >> $LOG_FILE
        return 0
    else
        echo "$(date): $SERVICE_NAME is not running" >> $LOG_FILE
        return 1
    fi
}

restart_service() {
    echo "$(date): Restarting $SERVICE_NAME" >> $LOG_FILE
    systemctl restart $SERVICE_NAME
    sleep 5
    
    if check_service; then
        echo "$(date): $SERVICE_NAME restarted successfully" >> $LOG_FILE
    else
        echo "$(date): Failed to restart $SERVICE_NAME" >> $LOG_FILE
    fi
}

# Main logic
if ! check_service; then
    restart_service
fi
```

### Scheduled Monitoring

```bash
# Add to crontab
* * * * * /usr/local/bin/monitor.sh
```

### Log Rotation

Create `/etc/logrotate.d/shortlinker`:

```
/opt/shortlinker/logs/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    postrotate
        systemctl reload shortlinker
    endscript
}
```

## Security Configuration

### Firewall Settings

```bash
# Allow local access only
sudo ufw allow from 127.0.0.1 to any port 8080

# Or allow reverse proxy server only
sudo ufw allow from 10.0.0.100 to any port 8080
```

### File Permissions

```bash
# Set correct permissions
sudo chown -R www-data:www-data /opt/shortlinker
sudo chmod 755 /opt/shortlinker
sudo chmod 600 /opt/shortlinker/data/links.db
sudo chmod 644 /opt/shortlinker/shortlinker
```

## SysV Init (Compatibility)

For systems that don't support systemd, use traditional init scripts:

```bash
#!/bin/bash
# /etc/init.d/shortlinker

case "$1" in
    start)
        echo "Starting shortlinker..."
        sudo -u www-data /opt/shortlinker/shortlinker &
        ;;
    stop)
        echo "Stopping shortlinker..."
        pkill -f shortlinker
        ;;
    restart)
        $0 stop
        sleep 2
        $0 start
        ;;
    *)
        echo "Usage: $0 {start|stop|restart}"
        exit 1
        ;;
esac
```
