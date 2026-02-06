# 系统服务配置

本页聚焦 systemd 原生服务部署。

## 文档导航

- [Docker Compose 与运维](/deployment/systemd-operations)

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

# 启动配置文件（必须）：
# - Shortlinker 会从 WorkingDirectory 读取 ./config.toml（相对路径）
# - 建议放在：/opt/shortlinker/config.toml
# - 需要配置的典型项：server.host/port、server.unix_socket、database.database_url、logging.*

# 安全配置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
# 需要写入：shortlinker.pid、./shortlinker.sock（IPC）、admin_token.txt（首次启动可选）、数据库文件等
ReadWritePaths=/opt/shortlinker

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


## 下一步

- [Docker Compose 与运维](/deployment/systemd-operations)
