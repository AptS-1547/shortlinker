# 系统服务配置：Docker Compose 与运维

本页聚焦 Docker Compose 生产服务、监控日志、安全配置与兼容模式。

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

### 服务管理

```bash
# 启动服务
docker-compose up -d

# 查看状态和日志
docker-compose ps
docker-compose logs -f shortlinker

# 停止和重启
docker-compose down
docker-compose restart shortlinker
```

## 监控和日志

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
    fi
}

# 主逻辑
if ! check_service; then
    restart_service
fi
```

### 定时监控

```bash
# 添加到 crontab
* * * * * /usr/local/bin/monitor.sh
```

### 日志轮转

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
        systemctl restart shortlinker
    endscript
}
```

## 安全配置

### 防火墙设置

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
sudo chmod 600 /opt/shortlinker/data/shortlinks.db
sudo chmod 755 /opt/shortlinker/shortlinker
```

## SysV Init（兼容性）

对于不支持 systemd 的系统，可以使用传统的 init 脚本：

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
