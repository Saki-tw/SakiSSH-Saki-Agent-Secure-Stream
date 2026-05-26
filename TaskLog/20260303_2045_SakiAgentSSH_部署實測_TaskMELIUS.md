# SakiAgentSSH 部署實測 TaskMELIUS

> 建立時間：20260303 2045 (UTC+8)
> 專案簡稱：SakiAgentSSH
> 原始任務：將 SakiAgentSSH daemon 實際部署至 Windows 節點

## 子任務一：Mac 本機編譯驗證 ✅

1. ✅ saki-ssh-daemon `cargo build --release` 成功（51s）
2. ✅ saki-ssh-client `cargo build --release` 成功（44s）
3. ✅ 本機 daemon↔client Ping: v1.0.0, OS=macos, Shell=bash
4. ✅ 本機 Execute: `echo 'hello from SakiAgentSSH'` 回傳正確
5. ✅ 進入交叉編譯

## 子任務二：Windows x86_64 交叉編譯 ✅

1. ✅ 確認 `x86_64-pc-windows-gnu` target 已安裝
2. ✅ 確認 `x86_64-w64-mingw32-gcc` (mingw-w64) 已安裝
3. ✅ Daemon 交叉編譯成功（50s）→ sakisshd.exe (16MB PE32+)
4. ✅ Client 交叉編譯成功（50s）→ sakissh.exe
5. ✅ `file` 驗證: PE32+ executable (console) x86-64, for MS Windows

## 子任務三：部署至 loser ✅

1. ✅ SSH 連通 loser (ssh saki@loser echo SSH_OK → OK)
2. ✅ 建立 `C:\SakiSSH\` 目錄
3. ✅ SCP sakisshd.exe → loser (16MB, 18.4MB/s)
4. ✅ SCP config.json → loser (ACL: 192.168.50.0/24 + 100.64.0.0/10)
5. ✅ 遠端啟動 daemon 成功

## 子任務四：gRPC 跨機驗證 ✅

1. ✅ Mac→loser Ping: v1.0.0, OS=windows, Shell=powershell (pwsh.exe 7.5.4)
2. ✅ Mac→loser Execute: `echo "Hello from loser"` 回傳正確
3. ✅ Mac→loser Execute: PowerShell 指令 PSVersion + Get-Date 正常
4. ⚠️ CJK 字元經 SSH conpty 傳遞有亂碼（daemon 端 gRPC UTF-8 stream 正確）
5. 🔶 常駐問題：SSH session 結束後 daemon 退出，需 daubl admin 設 Windows Service

## 子任務五：常駐安裝（待用戶協助）

1. ❌ `schtasks` 從 saki 帳號建立排程任務 → 權限不足（無輸出）
2. 需 daubl admin 以下擇一：
   - `sc create SakiAgentSSH binPath= "C:\SakiSSH\sakisshd.exe" start= auto`
   - 或 Task Scheduler GUI 建立系統啟動任務
3. 防火牆規則（若尚未開放 19284 port）：
   - `netsh advfirewall firewall add rule name=SakiAgentSSH dir=in action=allow protocol=TCP localport=19284`
