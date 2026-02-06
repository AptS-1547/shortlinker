# System Service Configuration

This page focuses on native systemd service deployment.

## Navigation

- [Docker Compose and Operations](/en/deployment/systemd-operations)

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

# Startup config file (required):
# - Shortlinker reads ./config.toml from WorkingDirectory (relative path)
# - Recommended location: /opt/shortlinker/config.toml
# - Typical keys: server.host/port, server.unix_socket, database.database_url, logging.*

# Security configuration
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
# Needs write access for: shortlinker.pid, ./shortlinker.sock (IPC), admin_token.txt (first startup), DB file, etc.
ReadWritePaths=/opt/shortlinker

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


## Next

- [Docker Compose and Operations](/en/deployment/systemd-operations)
