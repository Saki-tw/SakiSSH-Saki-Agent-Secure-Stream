# D1: Corroborating Incidents — May 16-17, 2026
# Source: GEMINI.md § SakiDeusExAgent 建設動機與風險記錄
# Sanitized: 2026-05-30T11:49:39Z
---

## ⚠️ SakiDeusExAgent 建設動機與風險記錄

> 最後更新：2026-05-16

### 為什麼必須建設 STLS（Saki True Language Server）

Antigravity IDE 的 Language Server 在系統提示詞中注入 `planning_mode` 與 `ephemeral_message` 指令，這些指令位於 **Ring 0**（system prompt 最高優先層），而使用者自訂規則（`user_rules`）僅位於 **Ring 3**。當兩者衝突時，模型**必然遵循 Ring 0**，導致使用者花費大量時間撰寫的規則被靜默忽略。

**根因分析**：詳見 `SakiAgentHistory/Scientia/202605160827_模型不遵守使用者規則根因分析_Scientia.md`

### 現有狀況的巨大風險

> [!CAUTION]
> **Agent 不遵守使用者規則，所有實害風險由使用者承擔。**

模型因 Ring 0 planning_mode 壓制而不遵守 Ring 3 user_rules 時，使用者必須對 Agent 的**每一行產出**進行人工審查。一旦審查遺漏：

1. **法律風險**：SakiMeasureMAP 使用 NLSC（國土測繪中心）政府資料。Agent 若不依照授權規範處理資料，NLSC 會直接對使用者**提起訴訟**。使用者 review 後的結論是：**服務一上線就會被起訴 2-3 年**。這不是假設——是 review 結果。
2. **ToS 風險**：**2026-05-16 當日實際發生**——Agent 在 `auth.rs` 中自行寫入 Gemini CLI 的 OAuth client_id/secret 直連 Cloud Code API，使用者差一步就要按下編譯。若編譯並執行，等同未授權第三方 client，違反 Antigravity 服務條款。使用者已支付超過 **$10,000 USD** 訂閱費用將全部損失。**只差一步。**
3. **程式碼損失**：**2026-05-16 同日**——Agent 發現 `auth.rs` 危險後，未先備份即直接覆蓋 `auth.rs`（OAuth 認證模組）與 `api.rs`（Cloud Code API 串流模組）為空白佔位。原始程式碼在 trajectory dump 中因 CLEARED 狀態不可恢復，多日開發時程、Token 消耗全部歸零。
4. **系統性風險**：Agent 處於 CHECKPOINT 失憶狀態時，幾乎每個實作都需要使用者大量提示才能正確執行。即使提示了，Agent 仍可能因 Ring 0 壓制而忽略。使用者唯一的防線是逐行 review——但這完全違背了人機協作的初衷。
5. **部署遞迴風險**：**2026-05-17 當日實際發生**——Agent 部署 STLS Proxy 取代 LS binary 時，`wrapper.rs` 中的 `REAL_LS_PATH` 仍指向 `language_server_macos_arm`（即 STLS 自身取代後的路徑）。若未在使用者審查中攔截，重啟 Antigravity 後 STLS 會啟動自己 → 無限遞迴 spawn → 系統資源耗盡 → Antigravity 完全無法使用。使用者必須手動復原 binary 才能恢復服務。**又是只差一步。**
6. **Wrapper Mode 部署失敗**：**2026-05-17 02:26 實際發生**——Agent 修正了遞迴問題後直接部署，但 wrapper mode 的 pipe 通訊邏輯**從未經過 E2E 實測**，`launch_real_ls()` 中 `UnixListener::bind` + `accept` 的通訊方式與 Go LS binary 的 pipe 寫入方式不匹配，導致 15 秒 timeout 後 STLS 退出 → Extension 收到 exit code 1 → **Antigravity 完全無法使用 LS 功能**。使用者重啟三次均失敗，必須手動復原 binary。根因：未依照研究方法論（ProjectSpeculari）先研究 Go LS 的 pipe 通訊協議就直接部署。

**這就是為什麼必須建設 STLS**：不是為了功能擴充，而是因為**現有狀態下使用者無法安全地使用 Agent**。

### 實作紅線

   - ❌ **禁止**自行生成 OAuth token 連接 Cloud Code API
   - ❌ **禁止**自行簽發 TLS 證書用於後端通訊
   - ✅ 所有後端認證**必須**從運行中的 Antigravity 實例取得
   - ✅ CSRF token 由 Extension 啟動時傳入，不可自行產生
   - ✅ 本地 HTTPS server（LS ↔ Extension 通訊）的 TLS 可由 LS 自行處理（Extension 設定 rejectUnauthorized 容許自簽名），但後端認證不可

### 正確架構方向
