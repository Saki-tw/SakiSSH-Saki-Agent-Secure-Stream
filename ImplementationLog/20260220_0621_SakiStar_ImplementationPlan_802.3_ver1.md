# SakiStar IPv6 雙通道並行架構

## 問題

M1 Mac 連 Router 有兩條通道（802.11ac Wi-Fi + 2.5GbE USB-to-RJ45），但目前：
- 僅 Wi-Fi 在用，2.5GbE 的 en5 未掛載
- SSH 配置混用 IPv4（192.168.50.x），隨著介面切換可能 IP 錯亂
- 雙通道無法並行利用

## 目標

以 IPv6 ULA 統一定址，讓 Wi-Fi 和有線 LAN 雙通道並行，消除 IPv4 IP 錯亂。

## 已確認的 IPv6 環境

| 項目 | 值 |
|------|-----|
| Router ULA 前綴 | `fd1d:98b6:21d3::/64`（via en1） |
| ISP GUA 前綴 | `2001:b011:200c:5d6f::/64` |
| Mac en1 ULA | 需確認（目前 NDP 表有 `fd1d` 路由） |
| Tailscale | `fd7a:115c:a1e0::/48`（utun4，之前被誤記為 Router 的 fd7b） |
| Loser SSHD | ❌ 只聽 IPv4（`AddressFamily inet`） |
| Trader SSHD | ✅ 預設聽 IPv4+IPv6 |

> [!IMPORTANT]
> 之前文檔中的 `fd7b:7026:9be3:43eb::/64` 是 **Tailscale 的 ULA**，不是 Router 的。
> Router 實際 ULA 前綴是 `fd1d:98b6:21d3::/64`。

---

## Proposed Changes

### 1. 修復 en5 2.5GbE 轉接器

#### 診斷

en5 (USB 10/100/1G/2.5G LAN) 有插有通電但 macOS 未列舉。`SPUSBDataType` 完全無輸出。

#### 行動

```bash
# 重新觸發 USB 列舉
sudo kextunload -b com.apple.driver.usb.cdc.ecm
sudo kextload -b com.apple.driver.usb.cdc.ecm

# 如果不行，嘗試重置 Hub USB bus
# 拔插 Hub 的 USB-C 連接（或拔插轉接器本身）

# 驗證
ifconfig en5 2>/dev/null && echo "en5 掛載成功" || echo "仍未掛載"
```

> [!WARNING]
> 如果 en5 始終無法掛載，方案退化為「Wi-Fi 單通道 + IPv6 統一定址」，仍然有價值（消除 IP 錯亂）。

---

### 2. 啟用 Loser SSHD IPv6

需要 Loser 管理員帳號 `daubl` 執行：

```powershell
# 修改 C:\ProgramData\ssh\sshd_config
# 改 AddressFamily inet → AddressFamily any
# 改 ListenAddress 0.0.0.0 → 加上 ListenAddress ::
Restart-Service sshd
```

---

### 3. 查詢各節點 IPv6 ULA 地址

```bash
# Mac 端
ifconfig en1 | grep "fd1d"

# Loser（SSH 後）
ssh saki@192.168.50.82 "ipconfig | findstr fd1d"

# Trader
ssh saki@192.168.50.10 "ipconfig | findstr fd1d"
```

---

### 4. 更新 SSH Config 為 IPv6 優先

#### [MODIFY] `~/.ssh/config`

```diff
 Host loser
-    HostName 192.168.50.82
+    HostName fd1d:98b6:21d3::{loser-suffix}
+    # Fallback IPv4
+    # HostName 192.168.50.82
     User saki
+    AddressFamily inet6

 Host trader
-    HostName 192.168.50.10
+    HostName fd1d:98b6:21d3::{trader-suffix}
     User saki
+    AddressFamily inet6

 Host loser-v4
     HostName 192.168.50.82
     User saki
+    AddressFamily inet

 Host loser-ts
     HostName 100.68.112.76
     User saki
```

> IPv6 ULA 的優勢：**無論 Mac 走 en1(Wi-Fi) 還是 en5(2.5GbE)**，
> 只要兩者都在 `fd1d:98b6:21d3::/64` 子網上，SSH 直接連目標 ULA。
> macOS 自動選擇 metric 最低（速度最快）的介面路由。

---

### 5. macOS 網路服務優先順序

```bash
# 查看目前順序
networksetup -listnetworkserviceorder

# 將 2.5GbE 排在 Wi-Fi 前面（如果 en5 修好）
sudo networksetup -ordernetworkservices \
    "forIP" \
    "USB 10/100/1G/2.5G LAN" \
    "Wi-Fi" \
    "Thunderbolt橋接器"
```

這讓 macOS 在路由決策時**優先走 en5 有線**，Wi-Fi 自動成為備援。

---

### 6. 更新文檔

#### [MODIFY] SakiMCP/SAKISTAR_CONNECTION_GUIDE.md
- 修正 IPv6 ULA 前綴（fd7b → fd1d）
- 新增雙通道並行架構說明
- 更新 SSH 配置範例

#### [MODIFY] SakiStarCommuncation/Scientia/20260214_0351_SakiStar_IPv6統一SSH架構_Scientia.md
- 修正 ULA 前綴（fd7b 是 Tailscale，fd1d 才是 Router）
- 更新 Loser/Trader IPv6 地址

---

## Verification Plan

### 自動驗證

```bash
# 1. 確認 en5 狀態
ifconfig en5 | grep "inet6"

# 2. 確認 IPv6 到 Loser 可達
ping6 -c 3 fd1d:98b6:21d3::{loser-suffix}%en1

# 3. 確認 SSH 走 IPv6
ssh -v loser 2>&1 | grep "Connecting to"

# 4. 確認雙通道路由
traceroute6 fd1d:98b6:21d3::{loser-suffix}
```

### 手動驗證（需用戶協助）

1. Loser 管理員帳號 `daubl` 修改 `sshd_config` 並重啟 SSHD
2. 用戶確認 Loser/Trader 是否開機在線
3. 拔插 USB 2.5GbE 轉接器測試 en5 是否自動恢復
