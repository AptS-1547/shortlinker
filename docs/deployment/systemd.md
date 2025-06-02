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

# 环境变量 - TCP 端口
Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080

# 环境变量 - Unix 套接字（二选一）
# Environment=UNIX_SOCKET=/tmp/shortlinker.sock

Environment=STORAGE_BACKEND=sqlite
Environment=DB_FILE_NAME=/opt/shortlinker/data/links.db
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

# 启动/停止/重启服务
sudo systemctl start shortlinker
sudo systemctl stop shortlinker
sudo systemctl restart shortlinker

# 查看日志
sudo journalctl -u shortlinker -f
sudo journalctl -u shortlinker --since "1 hour ago"
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
    environment:
      - SERVER_HOST=0.0.0.0
      - STORAGE_BACKEND=sqlite
      - DB_FILE_NAME=/data/shortlinker.data
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
        systemctl reload shortlinker
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
sudo chmod 600 /opt/shortlinker/data/links.db
sudo chmod 644 /opt/shortlinker/shortlinker
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
