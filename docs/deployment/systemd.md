# 系统服务配置

将 Shortlinker 配置为系统服务，实现开机自启和服务管理。

## systemd 服务

### 创建服务文件

创建 `/etc/systemd/system/shortlinker.service`：

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

# 环境变量
Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080
Environment=LINKS_FILE=/opt/shortlinker/data/links.json
Environment=DEFAULT_URL=https://example.com
Environment=RUST_LOG=info

# 安全配置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/shortlinker/data

[Install]
WantedBy=multi-user.target
```

### 部署步骤

```bash
# 创建用户和目录
sudo useradd --system --shell /bin/false --home /opt/shortlinker www-data
sudo mkdir -p /opt/shortlinker/{data,logs}
sudo chown -R www-data:www-data /opt/shortlinker

# 复制二进制文件
sudo cp shortlinker /opt/shortlinker/
sudo chmod +x /opt/shortlinker/shortlinker

# 安装并启动服务
sudo systemctl daemon-reload
sudo systemctl enable shortlinker
sudo systemctl start shortlinker
```

### 服务管理

```bash
# 查看状态
sudo systemctl status shortlinker

# 启动服务
sudo systemctl start shortlinker

# 停止服务
sudo systemctl stop shortlinker

# 重启服务
sudo systemctl restart shortlinker

# 查看日志
sudo journalctl -u shortlinker -f

# 查看最近日志
sudo journalctl -u shortlinker --since "1 hour ago"
```

## SysV Init（CentOS 6/RHEL 6）

### 创建启动脚本

创建 `/etc/init.d/shortlinker`：

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

### 安装和配置

```bash
# 安装启动脚本
sudo chmod +x /etc/init.d/shortlinker
sudo chkconfig --add shortlinker
sudo chkconfig shortlinker on

# 启动服务
sudo service shortlinker start
```

## Docker Compose 服务

### 生产环境配置

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
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/"]
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

### 服务管理

```bash
# 启动服务
docker-compose up -d

# 查看状态
docker-compose ps

# 查看日志
docker-compose logs -f shortlinker

# 停止服务
docker-compose down

# 重启服务
docker-compose restart shortlinker
```

## 监控和告警

### 服务监控脚本

```bash
#!/bin/bash
# monitor.sh - 服务监控脚本

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
        # 发送告警通知
        echo "$SERVICE_NAME restart failed on $(hostname)" | mail -s "Service Alert" admin@example.com
    fi
}

# 主逻辑
if ! check_service; then
    restart_service
fi
```

### 添加到 crontab

```bash
# 每分钟检查一次服务状态
* * * * * /usr/local/bin/monitor.sh
```

## 日志轮转

### logrotate 配置

创建 `/etc/logrotate.d/shortlinker`：

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

## 安全加固

### 防火墙配置

```bash
# 只允许本地访问
sudo ufw allow from 127.0.0.1 to any port 8080

# 或者只允许反向代理服务器访问
sudo ufw allow from 10.0.0.100 to any port 8080
```

### 文件权限

```bash
# 设置正确的权限
sudo chown -R www-data:www-data /opt/shortlinker
sudo chmod 755 /opt/shortlinker
sudo chmod 600 /opt/shortlinker/data/links.json
sudo chmod 644 /opt/shortlinker/shortlinker
```
