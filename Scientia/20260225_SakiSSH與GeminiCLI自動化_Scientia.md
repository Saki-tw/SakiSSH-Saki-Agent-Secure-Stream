# SakiSSH 與 Gemini CLI 跨機自動化研究 (Axiom 預研)

> **建立時間**：2026-02-25 20:30 (UTC+8)
> **標籤**：#SakiSSH #GeminiCLI #自動化 #Axiom

## 核心願景
透過 SakiSSH 的 gRPC 高效通道，實現 Control Plane (M1) 對 Compute Plane (Windows) 上 Gemini CLI 的直接調用。這將允許 Agent 在不同機器間傳遞複雜指令與狀態。

## 對接標準：Nushell Only
依照機構指令，所有透過 SakiSSH 在 Windows 上執行的指令必須且僅能對接 **Nushell**。這確保了跨平台腳本的一致性與強大的管線處理能力。

## 實作路徑
1. **路徑鎖定**: Daemon 內部指令寫死為 `C:\Program Files
uin
u.exe -c`。
2. **Gemini 整合**: 
   - 確保 Windows 端的 `gemini` 執行檔路徑已加入 Nushell 的 `$env.PATH`。
   - 透過 `sakissh` 傳送 `gemini` 指令，Daemon 會將其包裝為 `nu -c "gemini ..."`。

## 預期效益
- **零亂碼**: Nushell 與 gRPC 均原生支援 UTF-8，徹底解決 Windows 下的 ANSI/GBK 亂碼問題。
- **跨機 Agent 協作**: M1 上的 Agent 可以呼叫 Windows 上的 Agent 執行編譯、測試或日誌分析，並獲取流式輸出。
