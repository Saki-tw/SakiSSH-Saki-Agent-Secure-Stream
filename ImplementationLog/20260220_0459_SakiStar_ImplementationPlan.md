# 實作計畫歸檔 - SakiClip部署與網路排查

> 建立時間：2026-02-20 04:59 (UTC+8)

## 任務目標
1. 解決 Trader 遠端連接 SakiClip 以及安全疑慮。
2. 更新 Loser 與 Trader 的 SakiClip 至最新版。
3. 解決 M1 Mac 透過 TB4 Hub 到 Loser/Trader 的 USB 10Gbps 專線連線問題。

## 執行步驟
1. **SakiClip 部署**
   - 透過 SSH SCP 將最新二進位檔部署至 Loser/Trader。
   - 解決 dotnet build 在 SSH Session 的權限問題 (`DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1`)。
   - 刪除舊版檔案，建立附帶馬卡龍紫至勿忘草青漸變 Icon 的桌面捷徑。
2. **網路/硬體排查**
   - 調查 M1 `bridge0` 狀態不活躍的原因。
   - 分析 Loser (i5-12500H) 與 Trader (i5-8265U) 的 PCI 設備樹，確認兩台都沒有 Thunderbolt/USB4 controllers。
   - 發現 Loser 的 UCM-UCSI 裝置，但缺乏 URS（USB Role Switch）以致無法支援 Device Mode（DRD）。
   - 結論：TB4 Hub 的下游端口只能作為 Host，無法建立 Host-to-Host CDC-ECM 連線。
   - 最終解決方案：必須使用附帶 ASIC 橋接晶片的 **USB Bridge Cable（如 JUC501）** 來連通兩端。
3. **文件撰寫**
   - 撰寫 `SAKISTAR_CONNECTION_GUIDE.md`（SakiMCP 目錄）。
   - 撰寫 `20260220_0324_SakiStar_SakiClip版本與10Gbps排查報告_Scientia.md`（Scientia 目錄）。

## 後續行動
- 購買 JUC501 線材並連接設備，自動取得 IPv6 link-local IP 後修改 SSH Config 走該線路。
