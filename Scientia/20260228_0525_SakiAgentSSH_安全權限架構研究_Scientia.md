# SakiAgentSSH 安全權限架構研究

> **建立時間**：20260228_0525 (UTC+8)
> **標籤**：#SakiAgentSSH #安全 #權限架構 #研究

---

## 一、OpenSSH 既有權限模型

### 核心安全架構（三層協議）
1. **Transport Layer**：加密通道建立、演算法協商、伺服器認證
2. **User Authentication Layer**：支援密碼、公鑰、host-based、challenge-response
3. **Connection Layer**：多工通道、TCP 轉發、流量控制

### 權限隔離機制
- **Privilege Separation**：`sshd` 啟動後分離出非特權進程處理網路流量
- **Chroot**：透過 `ChrootDirectory` 將使用者限制在特定目錄
- **存取控制**：`AllowUsers`/`DenyUsers`/`AllowGroups`/`DenyGroups` 指令

### 檔案權限要求
| 路徑 | 權限 | 說明 |
|------|------|------|
| `~/.ssh/` | 700 | 僅擁有者可存取 |
| 私鑰 (`id_ed25519`) | 600 | 僅擁有者可讀寫 |
| 公鑰 / `authorized_keys` | 644 | 擁有者讀寫、其他人唯讀 |

---

## 二、SakiAgentSSH 在 Windows 的權限架構

### install.ps1 -CreateUser 專屬帳號策略
SakiAgentSSH 的 `install.ps1` 安裝腳本應遵循以下最佳實踐：

1. **最小權限原則 (PoLP)**
   - `saki` 帳號僅授予執行 daemon 所需的最低權限
   - 禁止 Administrator 等級權限
   - 定期審查權限範圍

2. **專屬帳號隔離**
   - 每個服務使用唯一帳號（`saki_sakisshd`）
   - 單一帳號遭入侵不影響其他服務
   - 命名規範：`Saki_ServiceFunction`

3. **登入限制**
   - 禁止互動式登入（Group Policy 限制）
   - 帳號僅用於服務執行
   - 密碼策略：強密碼 + 定期輪換（或使用 gMSA 自動管理）

4. **Windows 特有考量**
   - 可考慮 Managed Service Account (MSA) 或 Group MSA (gMSA)
   - `NT Authority\SYSTEM` 對 `authorized_keys` 具有完全控制權
   - 使用 `BUILTIN\Administrators` 安全群組管理

### 建議的 install.ps1 權限配置
```powershell
# 建立專屬帳號（最小權限）
New-LocalUser -Name "saki_sakisshd" -Description "SakiAgentSSH Daemon" -NoPassword
# 僅授予 "Log on as service" 權限
# 禁止互動式登入
# 服務安裝時指定此帳號
```

---

## 三、SakiAgentSSH 在 macOS 的權限架構

### LaunchDaemon vs LaunchAgent 差異

| 特性 | LaunchDaemon | LaunchAgent |
|------|-------------|-------------|
| 執行身份 | root | 登入使用者 |
| 載入時機 | 系統開機 | 使用者登入 |
| GUI 互動 | ❌ 不可 | ✅ 可 |
| .plist 路徑 | `/Library/LaunchDaemons/` | `~/Library/LaunchAgents/` |
| 安全風險 | 高（root 權限） | 低（使用者權限） |
| 持續執行 | 是（不受登出影響） | 否（隨使用者登出停止） |

### SakiAgentSSH 建議配置

**Daemon（建議使用 LaunchDaemon）**：
- 以 LaunchDaemon 部署於 `/Library/LaunchDaemons/tw.saki.sakisshd.plist`
- 透過 `UserName` 鍵降權至專屬使用者（非 root 直接執行）
- .plist 權限：`root:wheel` 644
- 使用 XPC 與使用者空間應用溝通

**Client（建議使用 LaunchAgent 或手動執行）**：
- 放置於 `~/Library/LaunchAgents/tw.saki.sakissh.plist`（若需自動啟動）
- 或由使用者手動呼叫

---

## 四、ED25519 金鑰交換機制整合

### 現階段架構可行性

SakiAgentSSH 目前使用 gRPC over TCP，金鑰認證可作為未來安全增強：

| Rust Crate | 用途 | 推薦度 |
|-----------|------|--------|
| `ed25519-dalek` | 金鑰生成與簽名驗證 | ⭐⭐⭐ 首選 |
| `ssh_key` | SSH 金鑰格式解析（Pure Rust） | ⭐⭐⭐ 格式相容 |
| `openssh-keys` | 公鑰解析 | ⭐⭐ 輔助 |

### 整合架構設計
```
┌─────────────────────────────────────────────────┐
│            SakiAgentSSH 認證流程                  │
│                                                   │
│  Client               Daemon                     │
│  ├─ 載入 id_ed25519  ├─ 載入 authorized_keys     │
│  ├─ 使用私鑰簽名     ├─ 驗證簽名                  │
│  ├─ 建立 gRPC 通道   ├─ 授權存取                  │
│  └─ config.json 指定  └─ 日誌記錄                  │
│       金鑰路徑                                     │
└─────────────────────────────────────────────────┘
```

### 實作建議（未來版本 v0.3.0）
1. 在 `config.json` 新增 `auth` 區塊指定金鑰路徑
2. 首次連線時進行 ED25519 Challenge-Response
3. 成功後建立 gRPC 加密通道
4. 支援 `~/.sakissh/authorized_keys` 格式

---

## 五、SakiAgentSSH 安全最佳實踐指南

### 部署清單

1. **✅ 最小權限** — 永遠使用專屬帳號執行 daemon
2. **✅ 網路隔離** — 預設僅綁定 `127.0.0.1`，透過 `config.json` CIDR 白名單控制
3. **✅ 禁止互動登入** — 服務帳號不允許互動式登入
4. **✅ 日誌審計** — 所有操作記錄含 UUID、時間戳、來源 IP
5. **✅ 金鑰認證**（v0.3.0）— 導入 ED25519 取代明文信任
6. **✅ 定期更新** — 透過套件管理器自動更新
7. **✅ 檔案權限** — 設定檔與金鑰具備正確權限

### 已知風險與緩解
| 風險 | 嚴重度 | 緩解方式 |
|------|--------|---------|
| 未認證 gRPC 連線 | 中 | CIDR 白名單 + 未來金鑰認證 |
| Daemon 以 root 執行 | 高 | 降權至專屬帳號 |
| config.json 洩漏 | 低 | 設定檔權限 600 |
