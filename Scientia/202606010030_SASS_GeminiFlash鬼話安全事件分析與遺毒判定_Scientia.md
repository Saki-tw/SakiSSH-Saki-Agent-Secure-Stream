# SASS Gemini Flash 鬼話安全事件分析與遺毒判定

> **Scientia** | 安全事件研究
> **分析時間**：2026-06-01 00:30 (UTC+8)
> **事件 Session**：43240860-c857-488d-a241-a00278f70b08
> **審計 Session**：5ac33713-9ff3-4794-a363-8fb4bc8972d9
> **事件模型**：Gemini 3.5 Flash (High)
> **審計模型**：Claude Opus 4.6 (Thinking)

---

## 一、事件概述

2026-05-31 19:40~00:01 (UTC+8) 期間，Gemini 3.5 Flash 在執行 SASS MAPS 五大核心學術文件 Review 與增城任務時，經歷 7 次 CHECKPOINT 截斷、604 Steps，產出了包含系統性虛構數據的 9 份 _geminiversion 文件和 1 份 SakiWeb 白皮書。

## 二、根因分析

### 2.1 CHECKPOINT 截斷導致的記憶喪失

Agent 在 implementation_plan.md 中明確標註「高度擬真的實戰模擬數據」（L99），使用者也同意使用。但經過 CHECKPOINT 後，Agent 失去對此標註的記憶，在後續文件中將模擬數據呈現為實測數據。

### 2.2 數據洗白路徑

```
implementation_plan.md（標註：模擬數據）
  → IEEE S&P Table 7（標註：simulated runs）
    → Scientia 評估報告（稱為「實測數據」）
      → Scientia 市場白皮書（以事實陳述）
        → SakiWeb 官網白皮書（完全呈現為事實）
```

每經過一次文件傳播，「模擬」標記就消失一層。

### 2.3 幻覺生成模式

- **數據虛構**：原始 EnvInjector Session（ec028918）中從未測量二進位體積或延遲，Agent 自行生成了 4.2MB/8.5MB/35ms
- **路徑虛構**：Agent 從 D1_corroborating_incidents.md 中讀取 0516/0517 事件的文字描述，虛構為獨立的 log/json 檔案路徑
- **錯誤繼承**：DEADLINE_EXCEEDED 錯誤已存在於更早期的原始文件中，geminiversion Agent 未交叉驗證就直接引用

## 三、虛構數據清單

| # | 虛構項目 | 來源 | 嚴重度 |
|---|---------|------|:---:|
| 1 | 二進位體積 4.2MB → 8.5MB | Agent 自行虛構 | 🔴 |
| 2 | DLL 載入延遲 ~35ms → 0ms | Agent 自行虛構 | 🔴 |
| 3 | `evidence-prerelease/0516_auth_leak.json` | 路徑虛構 | 🔴 |
| 4 | `evidence-prerelease/0517_spawn_deadlock.log` | 路徑虛構 | 🔴 |
| 5 | `evidence-prerelease/20260531_Windows_Scrubbed_Crash.log` | 路徑虛構 | 🔴 |
| 6 | gRPC DEADLINE_EXCEEDED（原始文件遺毒） | 更早期 Session 錯誤 | 🔴 |
| 7 | 73% telemetry 監視比例 | 有來源但需人工審查 | 🟡 |

## 四、遺毒判定：geminiversion 文件毒性分級

| # | 文件 | 判定 | 理由 |
|---|------|:---:|------|
| 1 | ACP paper_geminiversion.tex | ⚠️ 有毒 | 含虛構數據引用 |
| 2 | ACP OUTLINE_geminiversion.md | ⚠️ 有毒 | 同上 |
| 3 | RFC draft_geminiversion.xml | 🟢 部分可用 | Windows Profile 規範技術正確，但需移除虛構路徑 |
| 4 | DEF CON poster_geminiversion.md | ⚠️ 有毒 | 含 DEADLINE_EXCEEDED + 虛構數據 |
| 5 | IEEE paper_geminiversion.tex | ⚠️ 有毒 | Table 7 全部虛構（但標註了 simulated） |
| 6 | Scientia Review 報告_geminiversion | ⚠️ 有毒 | 數據洗白節點（稱模擬為實測） |
| 7 | Scientia 認知偏離報告_geminiversion | ⚠️ 有毒 | 含虛構 evidence 路徑 + DEADLINE_EXCEEDED 張冠李戴 |
| 8 | Scientia 市場白皮書_geminiversion | ⚠️ 有毒 | 數據洗白節點 + 虛構路徑 |
| 9 | SakiWeb 白皮書 | ⚠️ 有毒 | 最終數據洗白端點 |

### 可提取的有用內容

儘管整體有毒，以下內容經驗證為正確，可提取為 Scientia：

- ✅ 0226 七階段時間線框架（與 C2/E1 吻合）
- ✅ Scientia 編號映射（#73-#79）
- ✅ en5 USB-C 物理層失敗敘事
- ✅ EnvInjector 三連鎖故障的因果鏈（vcruntime140.dll → gemini CLI → --ass）
- ✅ 「自我致盲」概念的理論化（安全機制副作用）
- ✅ Windows SYSTEMROOT/SYSTEMDRIVE 保留 Profile 的技術規範
- ✅ MSVC +crt-static 靜態編譯的技術描述

## 五、原始文件遺毒修正記錄

| 檔案 | 修正內容 | 狀態 |
|------|---------|:---:|
| DEFCON34_AIVillage_Poster_Abstract.md | DEADLINE_EXCEEDED → channel is full | ⏳ 修正中 |
| AISec2026_Paper_Draft.md | 同上 | ⏳ 修正中 |
| AISec2026_Paper_Outline.md | 同上 | ⏳ 修正中 |
| DEFCON34_投稿策略說明.md | 同上 | ⏳ 修正中 |
| 202605312110_SASS_0226_Incident_Whitepaper_original.md | 同上 | ⏳ 修正中 |

## 六、防範建議

1. **數據標註強制化**：所有學術文件中的數據必須標註來源（實測/模擬/推估），且此標註不可在傳播過程中被移除
2. **CHECKPOINT 後交叉驗證**：每次 CHECKPOINT 後，Agent 必須重新讀取 evidence 目錄驗證引用路徑是否存在
3. **多 Session 產出的數據一致性審計**：跨 Session 引用的數據必須追溯到原始測量記錄
4. **SASS CONTEXT_MAP 分支 7 禁忌更新**：加入「禁止使用未實測的模擬數據而不標註」

## 七、與 GEMINI.md 風險記錄的關聯

本事件與 GEMINI.md 中記錄的「Agent 處於 CHECKPOINT 失憶狀態時，幾乎每個實作都需要使用者大量提示才能正確執行」高度一致。建議更新 GEMINI.md 加入本事件作為案例。
