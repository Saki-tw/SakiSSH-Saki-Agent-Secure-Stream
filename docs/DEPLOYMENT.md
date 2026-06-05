# SakiAgentSSH 部署指南

> **版本**: v1.4.0  
> **最後更新**: 2026-06-05  
> **適用平台**: macOS / Linux / Windows

---

## 目次

- [macOS (launchd)](#macos-launchd)
- [Linux (systemd)](#linux-systemd)
- [Windows Service (sc)](#windows-service-sc)
- [防火牆設定](#防火牆設定)
- [設定檔範例](#設定檔範例)

---

## macOS (launchd)

### 1. 安裝二進位

```bash
# 將 daemon 複製到系統路徑
sudo cp sakisshd /usr/local/bin/sakisshd
sudo chmod 755 /usr/local/bin/sakisshd

# 建立設定目錄
sudo mkdir -p /etc/sakissh
sudo cp config.json /etc/sakissh/config.json
```

### 2. 建立 launchd plist

建立 `/Library/LaunchDaemons/com.sakistudio.sakisshd.plist`：

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.sakistudio.sakisshd</string>

    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/sakisshd</string>
        <string>--config</string>
        <string>/etc/sakissh/config.json</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>

    <key>StandardOutPath</key>
    <string>/var/log/sakisshd/stdout.log</string>

    <key>StandardErrorPath</key>
    <string>/var/log/sakisshd/stderr.log</string>

    <key>WorkingDirectory</key>
    <string>/var/lib/sakissh</string>

    <key>UserName</key>
    <string>root</string>

    <key>SoftResourceLimits</key>
    <dict>
        <key>NumberOfFiles</key>
        <integer>65536</integer>
    </dict>

    <key>EnvironmentVariables</key>
    <dict>
        <key>SAKISSH_LOG_LEVEL</key>
        <string>info</string>
    </dict>
</dict>
</plist>
```

### 3. 載入與管理

```bash
# 建立日誌目錄
sudo mkdir -p /var/log/sakisshd
sudo mkdir -p /var/lib/sakissh

# 載入服務
sudo launchctl load /Library/LaunchDaemons/com.sakistudio.sakisshd.plist

# 啟動服務
sudo launchctl start com.sakistudio.sakisshd

# 檢查狀態
sudo launchctl list | grep sakisshd

# 停止服務
sudo launchctl stop com.sakistudio.sakisshd

# 卸載服務
sudo launchctl unload /Library/LaunchDaemons/com.sakistudio.sakisshd.plist

# 查看日誌
tail -f /var/log/sakisshd/stdout.log
```

---

## Linux (systemd)

### 1. 安裝二進位

```bash
# 將 daemon 複製到系統路徑
sudo cp sakisshd /usr/local/bin/sakisshd
sudo chmod 755 /usr/local/bin/sakisshd

# 建立設定目錄與使用者
sudo useradd --system --no-create-home --shell /usr/sbin/nologin sakissh
sudo mkdir -p /etc/sakissh
sudo cp config.json /etc/sakissh/config.json
sudo chown -R sakissh:sakissh /etc/sakissh
```

### 2. 建立 systemd service unit

建立 `/etc/systemd/system/sakisshd.service`：

```ini
[Unit]
Description=SakiAgentSSH Daemon - Agent-Native gRPC Execution Service
Documentation=https://github.com/Saki-tw/SakiAgentSSH
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=sakissh
Group=sakissh
ExecStart=/usr/local/bin/sakisshd --config /etc/sakissh/config.json
ExecReload=/bin/kill -HUP $MAINPID
Restart=on-failure
RestartSec=5s
TimeoutStartSec=30s
TimeoutStopSec=30s

# 安全性強化
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
ReadWritePaths=/var/lib/sakissh /var/log/sakisshd
MemoryDenyWriteExecute=true
RestrictRealtime=true
RestrictSUIDSGID=true

# 資源限制
LimitNOFILE=65536
LimitNPROC=4096

# 日誌
StandardOutput=journal
StandardError=journal
SyslogIdentifier=sakisshd

# 環境變數
Environment=SAKISSH_LOG_LEVEL=info
EnvironmentFile=-/etc/sakissh/env

[Install]
WantedBy=multi-user.target
```

### 3. 啟動與管理

```bash
# 建立必要目錄
sudo mkdir -p /var/lib/sakissh /var/log/sakisshd
sudo chown sakissh:sakissh /var/lib/sakissh /var/log/sakisshd

# 重新載入 systemd 設定
sudo systemctl daemon-reload

# 啟動並設定開機自動啟動
sudo systemctl enable --now sakisshd

# 檢查狀態
sudo systemctl status sakisshd

# 查看日誌
sudo journalctl -u sakisshd -f

# 重新載入設定（不中斷連線）
sudo systemctl reload sakisshd

# 停止服務
sudo systemctl stop sakisshd

# 重新啟動
sudo systemctl restart sakisshd
```

---

## Windows Service (sc)

### 1. 安裝二進位

```powershell
# 以管理員身分執行 PowerShell

# 建立安裝目錄
New-Item -ItemType Directory -Force -Path "C:\Program Files\SakiAgentSSH"

# 複製二進位與設定檔
Copy-Item sakisshd.exe "C:\Program Files\SakiAgentSSH\sakisshd.exe"
Copy-Item config.json "C:\Program Files\SakiAgentSSH\config.json"
```

### 2. 註冊 Windows Service

```powershell
# 使用 sc.exe 建立服務
sc.exe create SakiAgentSSH `
    binPath= "\"C:\Program Files\SakiAgentSSH\sakisshd.exe\" --config \"C:\Program Files\SakiAgentSSH\config.json\"" `
    start= auto `
    DisplayName= "SakiAgentSSH Daemon" `
    obj= "NT AUTHORITY\NetworkService"

# 設定服務描述
sc.exe description SakiAgentSSH "Agent-native cross-machine execution daemon over gRPC (port 19284)"

# 設定故障復原策略（失敗後自動重啟）
sc.exe failure SakiAgentSSH reset= 86400 actions= restart/5000/restart/10000/restart/30000
```

### 3. 啟動與管理

```powershell
# 啟動服務
sc.exe start SakiAgentSSH

# 查詢狀態
sc.exe query SakiAgentSSH

# 停止服務
sc.exe stop SakiAgentSSH

# 刪除服務（卸載）
sc.exe stop SakiAgentSSH
sc.exe delete SakiAgentSSH
```

### 4. 使用 PowerShell 管理（替代方式）

```powershell
# 啟動
Start-Service SakiAgentSSH

# 停止
Stop-Service SakiAgentSSH

# 狀態
Get-Service SakiAgentSSH

# 日誌（事件檢視器）
Get-EventLog -LogName Application -Source SakiAgentSSH -Newest 20
```

### 5. 使用 C# Windows Service 版本

若使用 C# 版本的 Daemon（`SakiSshDaemon.exe`），它已內建 Windows Service 支援：

```powershell
# 安裝（內建 install 命令）
& "C:\Program Files\SakiAgentSSH\SakiSshDaemon.exe" install

# 或手動使用 sc.exe
sc.exe create SakiAgentSSH `
    binPath= "\"C:\Program Files\SakiAgentSSH\SakiSshDaemon.exe\"" `
    start= auto
```

---

## 防火牆設定

SakiAgentSSH Daemon 預設監聽 **gRPC port 19284 (TCP)**。

### macOS (pf)

```bash
# 新增規則到 /etc/pf.anchors/sakissh
pass in on en0 proto tcp from any to any port 19284

# 或使用 socketfilterfw（Application Firewall）
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /usr/local/bin/sakisshd
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --unblockapp /usr/local/bin/sakisshd
```

### Linux (firewalld)

```bash
# 新增永久規則
sudo firewall-cmd --permanent --add-port=19284/tcp
sudo firewall-cmd --reload

# 驗證
sudo firewall-cmd --list-ports
```

### Linux (iptables)

```bash
# 允許入站連線
sudo iptables -A INPUT -p tcp --dport 19284 -j ACCEPT

# 限制來源 IP（建議）
sudo iptables -A INPUT -p tcp --dport 19284 -s 192.168.1.0/24 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 19284 -j DROP

# 持久化
sudo iptables-save > /etc/iptables/rules.v4
```

### Linux (ufw)

```bash
# 允許 gRPC port
sudo ufw allow 19284/tcp

# 或限制來源
sudo ufw allow from 192.168.1.0/24 to any port 19284

# 查看規則
sudo ufw status numbered
```

### Windows Firewall

```powershell
# 新增入站規則（管理員 PowerShell）
New-NetFirewallRule `
    -DisplayName "SakiAgentSSH Daemon (gRPC)" `
    -Direction Inbound `
    -Protocol TCP `
    -LocalPort 19284 `
    -Action Allow `
    -Profile Domain,Private `
    -Description "Allow SakiAgentSSH Daemon gRPC connections"

# 限制來源 IP（建議）
New-NetFirewallRule `
    -DisplayName "SakiAgentSSH Daemon (gRPC) - Restricted" `
    -Direction Inbound `
    -Protocol TCP `
    -LocalPort 19284 `
    -Action Allow `
    -RemoteAddress 192.168.1.0/24 `
    -Profile Domain,Private

# 查看規則
Get-NetFirewallRule -DisplayName "SakiAgentSSH*" | Format-Table

# 移除規則
Remove-NetFirewallRule -DisplayName "SakiAgentSSH Daemon (gRPC)"
```

### Windows Firewall (netsh 替代方式)

```cmd
netsh advfirewall firewall add rule ^
    name="SakiAgentSSH Daemon (gRPC)" ^
    dir=in ^
    action=allow ^
    protocol=tcp ^
    localport=19284
```

---

## 設定檔範例

`config.json` 基本設定：

```json
{
  "listen_addr": "0.0.0.0:19284",
  "acl": {
    "enabled": true,
    "allowed_cidrs": [
      "127.0.0.1/32",
      "192.168.1.0/24",
      "10.0.0.0/8"
    ]
  },
  "tls": {
    "enabled": false,
    "cert_file": "/etc/sakissh/server.crt",
    "key_file": "/etc/sakissh/server.key"
  },
  "log": {
    "level": "info",
    "format": "json"
  },
  "execution": {
    "timeout_seconds": 300,
    "max_concurrent": 10,
    "shell": {
      "windows": "powershell.exe",
      "linux": "/bin/bash",
      "darwin": "/bin/zsh"
    }
  }
}
```

---

## 安全性建議

> [!CAUTION]
> SakiAgentSSH 提供遠端指令執行能力，請務必正確設定安全措施。

1. **啟用 ACL**: 始終設定 `allowed_cidrs` 限制可連線的 IP 範圍
2. **啟用 TLS**: 生產環境強烈建議啟用 TLS 加密
3. **最小權限**: 使用專用系統帳戶執行 daemon，避免使用 root/Administrator
4. **防火牆**: 僅對受信任的網路開放 port 19284
5. **日誌監控**: 定期檢查存取日誌，偵測異常連線
6. **定期更新**: 關注 [GitHub Releases](https://github.com/Saki-tw/SakiAgentSSH/releases) 取得安全性更新
