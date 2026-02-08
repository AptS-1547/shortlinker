# System Service Configuration: Docker Compose and Operations

This page focuses on Docker Compose production setup, monitoring/logging, security configuration, and compatibility mode.

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
      - ./config.toml:/config.toml:ro
      - ./data:/data
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
        systemctl restart shortlinker
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
sudo chmod 600 /opt/shortlinker/data/shortlinks.db
sudo chmod 755 /opt/shortlinker/shortlinker
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
