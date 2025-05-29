# System Service Configuration

Configure Shortlinker as a system service for automatic startup and service management.

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

# Environment variables
Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080
Environment=LINKS_FILE=/opt/shortlinker/data/links.json
Environment=DEFAULT_URL=https://example.com
Environment=RUST_LOG=info
Environment=ADMIN_TOKEN=your_secure_production_token

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

# Start service
sudo systemctl start shortlinker

# Stop service
sudo systemctl stop shortlinker

# Restart service
sudo systemctl restart shortlinker

# View logs
sudo journalctl -u shortlinker -f

# View recent logs
sudo journalctl -u shortlinker --since "1 hour ago"
```

## SysV Init (CentOS 6/RHEL 6)

### Create Startup Script

Create `/etc/init.d/shortlinker`:

```bash
#!/bin/bash
# shortlinker        Shortlinker URL Shortening Service
# chkconfig: 35 99 99
# description: Shortlinker daemon

. /etc/rc.d/init.d/functions

USER="www-data"
DAEMON="shortlinker"
ROOT_DIR="/opt/shortlinker"

DAEMON_PATH="$ROOT_DIR/$DAEMON"
LOCK_FILE="/var/lock/subsys/shortlinker"

start() {
    if [ -f $LOCK_FILE ] ; then
        echo "$DAEMON is locked."
        return
    fi
    
    echo -n $"Starting $DAEMON: "
    runuser -l "$USER" -c "$DAEMON_PATH" && echo_success || echo_failure
    RETVAL=$?
    echo
    [ $RETVAL -eq 0 ] && touch $LOCK_FILE
    return $RETVAL
}

stop() {
    echo -n $"Shutting down $DAEMON: "
    pid=`ps -aefw | grep "$DAEMON" | grep -v " grep " | awk '{print $2}'`
    kill -9 $pid > /dev/null 2>&1
    [ $? -eq 0 ] && echo_success || echo_failure
    echo
    [ $? -eq 0 ] && rm -f $LOCK_FILE
}

restart() {
    stop
    start
}

status() {
    if [ -f $LOCK_FILE ]; then
        echo "$DAEMON is running."
    else
        echo "$DAEMON is stopped."
    fi
}

case "$1" in
    start)
        start
        ;;
    stop)
        stop
        ;;
    status)
        status
        ;;
    restart)
        restart
        ;;
    *)
        echo "Usage: {start|stop|status|restart}"
        exit 1
        ;;
esac

exit $?
```

### Installation and Configuration

```bash
# Install startup script
sudo chmod +x /etc/init.d/shortlinker
sudo chkconfig --add shortlinker
sudo chkconfig shortlinker on

# Start service
sudo service shortlinker start
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
      - ./logs:/logs
    environment:
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      - LINKS_FILE=/data/links.json
      - DEFAULT_URL=https://your-domain.com
      - RUST_LOG=info
      - ADMIN_TOKEN=${ADMIN_TOKEN}
      - ADMIN_ROUTE_PREFIX=/secure-admin
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:8080/"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    deploy:
      resources:
        limits:
          memory: 128M
          cpus: '0.5'
        reservations:
          memory: 64M
          cpus: '0.25'
    labels:
      - "com.docker.compose.service=shortlinker"
```

### Service Management

```bash
# Start services
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f shortlinker

# Stop services
docker-compose down

# Restart services
docker-compose restart shortlinker

# Update and restart
docker-compose pull && docker-compose up -d
```

## Monitoring and Alerting

### Service Monitoring Script

```bash
#!/bin/bash
# monitor.sh - Service monitoring script

SERVICE_NAME="shortlinker"
LOG_FILE="/var/log/shortlinker-monitor.log"
ADMIN_API_URL="http://localhost:8080/admin/link"
ADMIN_TOKEN="your_admin_token"

check_service() {
    if systemctl is-active --quiet $SERVICE_NAME; then
        echo "$(date): $SERVICE_NAME is running" >> $LOG_FILE
        return 0
    else
        echo "$(date): $SERVICE_NAME is not running" >> $LOG_FILE
        return 1
    fi
}

check_admin_api() {
    if [ -n "$ADMIN_TOKEN" ]; then
        response=$(curl -s -H "Authorization: Bearer $ADMIN_TOKEN" "$ADMIN_API_URL" || echo "error")
        if echo "$response" | grep -q '"code":0'; then
            echo "$(date): Admin API is responding" >> $LOG_FILE
            return 0
        else
            echo "$(date): Admin API is not responding properly" >> $LOG_FILE
            return 1
        fi
    fi
    return 0
}

restart_service() {
    echo "$(date): Restarting $SERVICE_NAME" >> $LOG_FILE
    systemctl restart $SERVICE_NAME
    sleep 5
    
    if check_service && check_admin_api; then
        echo "$(date): $SERVICE_NAME restarted successfully" >> $LOG_FILE
    else
        echo "$(date): Failed to restart $SERVICE_NAME" >> $LOG_FILE
        # Send alert notification
        echo "$SERVICE_NAME restart failed on $(hostname)" | mail -s "Service Alert" admin@example.com
    fi
}

# Main logic
if ! check_service; then
    restart_service
elif ! check_admin_api; then
    echo "$(date): Admin API check failed, restarting service" >> $LOG_FILE
    restart_service
fi
```

### Add to crontab

```bash
# Check service status every minute
* * * * * /usr/local/bin/monitor.sh

# Check Admin API health every 5 minutes
*/5 * * * * /usr/local/bin/check-admin-api.sh
```

### Health Check Script for Admin API

```bash
#!/bin/bash
# check-admin-api.sh - Admin API health check

ADMIN_API_URL="http://localhost:8080/admin/link"
ADMIN_TOKEN="your_admin_token"

if [ -z "$ADMIN_TOKEN" ]; then
    echo "Admin API disabled (no token set)"
    exit 0
fi

response=$(curl -s -w "%{http_code}" -H "Authorization: Bearer $ADMIN_TOKEN" "$ADMIN_API_URL")
http_code=${response: -3}

if [ "$http_code" = "200" ]; then
    echo "Admin API healthy"
    exit 0
else
    echo "Admin API unhealthy (HTTP $http_code)"
    exit 1
fi
```

## Log Rotation

### logrotate Configuration

Create `/etc/logrotate.d/shortlinker`:

```
/opt/shortlinker/logs/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 www-data www-data
    postrotate
        systemctl reload shortlinker || true
    endscript
}

/var/log/shortlinker*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    postrotate
        systemctl reload shortlinker || true
    endscript
}
```

## Security Hardening

### Firewall Configuration

```bash
# Allow only local access
sudo ufw allow from 127.0.0.1 to any port 8080

# Or allow only reverse proxy server access
sudo ufw allow from 10.0.0.100 to any port 8080

# For Admin API, be more restrictive
sudo ufw allow from 192.168.1.0/24 to any port 8080
```

### File Permissions

```bash
# Set correct permissions
sudo chown -R www-data:www-data /opt/shortlinker
sudo chmod 755 /opt/shortlinker
sudo chmod 600 /opt/shortlinker/data/links.json
sudo chmod 755 /opt/shortlinker/shortlinker

# Secure Admin API token in environment
sudo chmod 600 /opt/shortlinker/.env
```

### AppArmor Profile (Ubuntu/Debian)

Create `/etc/apparmor.d/shortlinker`:

```
#include <tunables/global>

/opt/shortlinker/shortlinker {
  #include <abstractions/base>
  
  # Allow network access
  network inet stream,
  network inet6 stream,
  
  # Allow reading configuration
  /opt/shortlinker/data/ r,
  /opt/shortlinker/data/** rw,
  
  # Allow log writing
  /opt/shortlinker/logs/ r,
  /opt/shortlinker/logs/** rw,
  
  # Deny everything else
  deny /etc/shadow r,
  deny /etc/passwd w,
  deny /home/** r,
  deny /root/** r,
}
```

### SELinux Policy (RHEL/CentOS)

```bash
# Create SELinux policy for Shortlinker
sudo setsebool -P httpd_can_network_connect 1
sudo semanage port -a -t http_port_t -p tcp 8080

# Set SELinux context for files
sudo semanage fcontext -a -t httpd_exec_t "/opt/shortlinker/shortlinker"
sudo semanage fcontext -a -t httpd_config_t "/opt/shortlinker/data(/.*)?"
sudo restorecon -R /opt/shortlinker
```

## Backup and Recovery

### Automated Backup Script

```bash
#!/bin/bash
# backup.sh - Automated backup script

BACKUP_DIR="/opt/backups/shortlinker"
DATA_FILE="/opt/shortlinker/data/links.json"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Backup data file
cp $DATA_FILE $BACKUP_DIR/links_$TIMESTAMP.json

# Compress old backups
find $BACKUP_DIR -name "links_*.json" -mtime +1 -exec gzip {} \;

# Remove backups older than 30 days
find $BACKUP_DIR -name "links_*.json.gz" -mtime +30 -delete

echo "Backup completed: $BACKUP_DIR/links_$TIMESTAMP.json"
```

### Recovery Procedure

```bash
# Stop service
sudo systemctl stop shortlinker

# Restore from backup
sudo cp /opt/backups/shortlinker/links_20240101_120000.json /opt/shortlinker/data/links.json
sudo chown www-data:www-data /opt/shortlinker/data/links.json

# Start service
sudo systemctl start shortlinker

# Verify recovery
curl -I http://localhost:8080/test-link
```
