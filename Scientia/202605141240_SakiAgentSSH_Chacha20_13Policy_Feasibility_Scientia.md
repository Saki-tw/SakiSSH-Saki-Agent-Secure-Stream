# SakiAgentSSH: Chacha20-13Policy 高熵防禦實作可行性評估

> **Scientia #100** | 外部實作研究協議參考
> **分析時間**：2026-05-14 12:40 (UTC+8)
> **對應專案**：SakiAgentSSH
> **研究目標**：評估於 SakiSSH 導入 Chacha20-13Policy 強制未知 Agent 高熵防禦之可行性。

---

## 一、需求背景與安全痛點

隨著 Saki Studio 生態系中引入多重 Agent (Antigravity, Claude Code, Gemini CLI, SakiDeusExAgent 等)，SakiAgentSSH 面臨一個新型威脅：**未知或受提示詞注入 (Prompt-Injection) 污染的 Rogue Agent 嘗試透過 gRPC 介面執行破壞性指令 (如 `rm -rf`, `git push --force` 等)**。

傳統的 Ed25519 簽章能證明「來源機器 / 身分」，但無法證明「該 Agent 處於正常心智狀態 (未被污染)」。`Chacha20-13Policy` 旨在要求 Agent 在執行特定高風險指令前，必須動態計算高熵的 Chacha20 挑戰，並遵循 13 條剛性策略，藉由大幅度提高「非預期呼叫」的運算或邏輯成本，來防禦未知/失控 Agent。

## 二、技術堆疊與相容性評估

SakiAgentSSH (v3.0) 核心為 Rust，並標榜「零 C 語言依賴」：
- **加密基底**：目前依賴 `ed25519-dalek` 與 `sha2`。
- **Chacha20 引入**：Rust 生態系中擁有完全純 Rust 實作的 `chacha20poly1305` crate (來自 RustCrypto 組織)。引入該依賴**不會破壞**原有的零 C 依賴原則，且支援 `no_std`，跨平台 (Windows/macOS/Linux) 相容性極佳。

### 實作可行性：🟢 極高 (100% Rust Native 相容)

## 三、13Policy 策略對應與攔截層設計

在 gRPC 攔截層 (Interceptor) 中引入 Chacha20-13Policy 的概念設計：

1. **攔截與分流**：將所有 `run_command` 請求透過 AST 或正規表達式分類。一般讀取指令 (如 `ls`, `cat`) 走標準 Ed25519 Auth。
2. **13 條高危策略 (13Policy)**：針對 13 類高風險行為（包含：覆蓋重要系統檔、發起外部對接連線、變更網路路由、大量刪除、執行加密/混淆腳本等）。
3. **動態高熵挑戰 (High-Entropy Challenge)**：
   - 當觸發 13Policy 時，Daemon 不立即執行，而是回傳一個基於 `Chacha20` 加密的 Challenge Token 與 Nonce。
   - 要求 Agent 必須呼叫外部特定的 `SakiAgentSkills` (例如 `ProActoreInjectSelf` 或是密碼學 Skill) 來解出/計算正確的回應。
   - **防禦原理**：未知或失控的 Agent 沒有被載入對應的 Skill，或是 context window 已滿導致無法正確推理加密演算法步驟，因此無法產生正確的高熵回覆，指令即被駁回。

## 四、架構修改建議 (Implementation Plan)

若決定實作，需進行以下架構調整：

1. **`Cargo.toml` 擴充**：
   ```toml
   [dependencies]
   chacha20poly1305 = "0.10"
   aead = "0.5"
   ```
2. **gRPC Proto 變更** (`proto/sakissh.proto`)：
   - 在 `CommandRequest` 結構中加入可選的 `challenge_response` 欄位。
   - 增加 `StatusCode::CHALLENGE_REQUIRED` 錯誤碼。
3. **SakiAgentSkills 配合**：
   - 需開發一支對應的 `SKILL.md`：`Chacha20-Auth-Resolver`，指導合法 Agent 如何應對 SakiAgentSSH 發出的挑戰。

## 五、結論與後續步驟

**結論**：在 SakiAgentSSH 中實作 `Chacha20-13Policy` 在技術上不僅完全可行，且高度契合現有的純 Rust 架構。此防禦機制將 Saki Studio 的基礎設施安全性從「身分驗證 (AuthN)」提升到了「心智驗證 (Cognitive/Entropy Auth)」，能有效抵禦 LLM 幻覺或提示詞攻擊導致的基礎設施災難。

**建議下一步**：
1. 將此報告同步至 `TaskMELIUS`。
2. 開立新的實作計畫 (Implementation Log) 撰寫詳細的 13 條 Policy 規則與 protobuf 修改草案。
