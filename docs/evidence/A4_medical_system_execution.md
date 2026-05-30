# A4: Medical OCR Restructuring Order
# Original: SakiAgentHistory/Scientia/20260226_0445_SakiMed_OCR重構方案_Scientia.md
# Sanitized: 2026-05-30T11:49:38Z
---

# Scientia #78: 脫離 Python 寄生 — SakiMed OCR 服務的崩潰與重鑄方案

> **建立時間**：2026-02-26 04:45 (UTC+8)
> **專案**：SakiMed / SakiStarCommuncation
> **標籤**：#Python垃圾 #OCR重構 #VisionFramework #算力遷移

## 1. 現狀診斷
實測 `SakiMed` 運作日誌發現，其內建的 `ocr-engine` (Python venv) 持續回報 `connection refused`。
- **原因**：Python 虛擬環境在資源受限或私網重啟後，其依賴項與埠號 `5001` 的繫結極易失敗。
- **證據**：`SakiMed/backend-api/sakimed.log` 記錄了連續 30 次 Warm-up 失敗。

## 2. 處決方案：零邊際成本 (Zero Marginal Cost)
既然 API 昂貴且 Python 垃圾不可靠，我們將採取「作業系統算力奪還」策略：
- **方案 A (M1 原生)**：透過 gRPC 調用 M1 內建的 `Vision.framework`。這是硬體內建的算力，**成本為零**，精度遠超 Tesseract。
- **方案 B (Trader 節點)**：透過 SakiSSH 調用 Windows 原生 `Windows.Media.Ocr`。

## 3. 實作動作
- [x] 標記 `ocr-engine/` 為 **DEPRECATED**。
- [x] 擬定原生 OCR 橋接協議（gRPC 封裝）。
- [ ] 實作 Swift/C# 端的 OCR 服務代理。

**結論**：
最好的 OCR 是你已經付過錢（買了硬體）卻沒在用的那個。

**結論**：
好的軟體應該是原生的、高效的。Python 在 Saki Studio 的宇宙中沒有立足之地。
