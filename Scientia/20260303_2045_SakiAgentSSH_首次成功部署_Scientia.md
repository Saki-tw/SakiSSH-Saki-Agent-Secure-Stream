# SakiAgentSSH 首次成功部署至 Windows

**時間**：2026-03-03 20:45 (TST)
**專案**：SakiAgentSSH
**里程碑**：首次跨機 gRPC 通訊驗證成功

---

## 部署環境

| 項目 | Mac (Hub) | loser (Node) |
|------|-----------|-------------|
| OS | macOS (M1) | Windows 11 |
| Shell | bash | PowerShell 7.5.4 (pwsh.exe) |
| 版本 | v1.0.0 | v1.0.0 |
| 路徑 | target/release/ | C:\SakiSSH\ |
| gRPC port | — | 19284 |

## ACL 配置

```json
{
  "allowed_cidrs": ["192.168.50.0/24", "100.64.0.0/10"]
}
```

- LAN (192.168.50.0/24): 內網直連
- Tailscale (100.64.0.0/10): VPN 備援

## 驗證結果

| 功能 | 狀態 | 備註 |
|------|------|------|
| Ping | ✅ | v1.0.0, uptime 回傳正常 |
| Execute | ✅ | PowerShell 指令正常執行 |
| UTF-8 | ⚠️ | CJK 經 SSH 有亂碼，gRPC stream 本身正確 |
| 常駐 | ❌ | SSH session 結束後 daemon 退出 |

## 下一步

1. **daubl 設定 Windows Service** 或排程任務讓 daemon 常駐
2. **防火牆規則** 永久開放 port 19284
3. **複製部署至 trading-v4**
4. **整合至 saki-orchestrator** heartbeat
