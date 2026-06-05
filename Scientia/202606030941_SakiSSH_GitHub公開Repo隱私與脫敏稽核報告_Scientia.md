# SakiAgentSSH GitHub 公開 Repo 隱私與脫敏稽核報告

> 生成時間：2026-06-03 09:41 (UTC+8)
> 稽核範圍：`Saki-tw/SakiSSH-Saki-Agent-Secure-Stream` 全部 git-tracked 檔案
> 前 Session：`1615c3d6-f383-4ef9-8773-40543abcdb0e`
> 當前 Session：`68f05f9a-f04a-4f62-b9f5-ad01f8019fd6`

---

## 一、概況

| 指標 | 數值 |
|------|------|
| Git-tracked 檔案總數 | **1,209** |
| Scientia/ 文件數 | **70** |
| TaskLog+ImplementationLog+WalkthroughLog 文件數 | **59** |
| 確認洩漏項目 | **4 項 🔴** |
| 待評估風險項目 | **8 項 🟡** |

---

## 二、Phase 2 — 四級分類

### 🔴 A 級：立即移除（密碼、Token、代號映射）

| # | 風險 | 位置 | 內容 | 行動 |
|---|------|------|------|------|
| A1 | **GitHub PAT token 外洩** | `.git/config` remote URL | `[REDACTED]` | 立即輪換 token + 改 credential helper |
| A2 | **明文密碼暴露** | `SASS_CHECKPOINT.md` §G (L742-746) | saki/9528, admin: [USER]/[REDACTED_PASS], 內網 IP | `git rm --cached` |
| A3 | **CODENAME_MAP 已 tracked** | `docs/evidence-prerelease/CODENAME_MAP.md` | 匿名化代號 ↔ 實名映射 | `git rm --cached`（.gitignore 已設但未 untrack） |
| A4 | **部署帳密** | `SASS_CHECKPOINT.md` §F (L693-746) | Windows 密碼、SSH 帳號、內網 IP 全表 | 同 A2 |

---

### 🟠 B 級：應移出 tracking

| # | 風險 | 檔案 | 原因 | 行動 |
|---|------|------|------|------|
| B1 | **CHECKPOINT 全文** | `SASS_CHECKPOINT.md` (38KB) | 含§F/§G 帳密、§B1.5 Reviewer 人格分析、§B4 IEEE 代號方案 | `git rm --cached` |
| B2 | **CONTEXT_MAP 全文** | `SASS_CONTEXT_MAP.Promissrum` (23KB) | 65 筆 `/Users/[USER]` 路徑暴露、完整分支路由 + 決策紀錄 | `git rm --cached` |
| B3 | **OmniaCorePrompt** | `20260525045821_SakiSSH_OmniaCorePrompt.Promissrum` | 內部 Agent 提示詞策略 | `git rm --cached` |
| B4 | **交棒上下文包** | `202605250407_SakiStar_交棒給Claude47的上下文包.md` | 內部 Session 交接文件 | `git rm --cached` |
| B5 | **全部 TaskLog/** | `TaskLog/` (34 檔) | 內部任務追蹤、含開發過程細節 | `git rm --cached -r` |
| B6 | **全部 ImplementationLog/** | `ImplementationLog/` (8 檔) | 內部實作計畫 | `git rm --cached -r` |
| B7 | **全部 WalkthroughLog/** | `WalkthroughLog/` (17 檔) | 內部實作歷程 | `git rm --cached -r` |
| B8 | **.agent/rules.md** | `.agent/rules.md` | Agent 工作規則（內部） | `git rm --cached -r` |

---

### 🟡 C 級：需脫敏處理（保留但移除敏感內容）

| # | 檔案 | 敏感內容 | 脫敏方法 |
|---|------|----------|----------|
| C1 | `Scientia/20260303_2045_SakiAgentSSH_首次成功部署_Scientia.md` | `allowed_cidrs: ["[INTERNAL_CIDR]", "100.64.0.0/10"]`、LAN IP | 替換為 `[REDACTED_CIDR]` |
| C2 | `Scientia/20260303_1118_機構匯流排完整情報與理想架構_Scientia.md` | 大量內網 IP（[INTERNAL_SUBNET]）、設備完整布局 | **建議移至 B 級**（內容幾乎全是內網架構） |
| C3 | `Scientia/20260224_2055_SakiStar_SakiSSH架構評估_Scientia.md` | 內網 IP [INTERNAL_IP] | 替換為 `[INTERNAL_IP]` |
| C4 | `Scientia/20260224_2130_SakiStar_基礎設施與SakiSSH標準_Promissrum.md` | 內網 IP、Tailscale IP 100.119.71.51、ACL 規則 | 替換為 `[INTERNAL_IP]`/`[TAILSCALE_IP]` |
| C5 | `Scientia/202605312025_SASS_核心架構報告與全防禦鏈安全實踐研究_Scientia.md` | 測試端點 IP [HOST_D] | 替換為 `[TEST_ENDPOINT]` |
| C6 | `Scientia/20260225_SakiSSH_Windows_Setup_Backup.ps1` | UAC 繞過配置、防火牆開放規則 | **建議移至 B 級**（部署腳本） |
| C7 | `Scientia/202605311542_遙測代號與論文匿名化映射_Scientia.md` | 可能含匿名化代號映射 | 需檢查是否重複 CODENAME_MAP |

---

### 🟢 D 級：安全可保留

以下類型的檔案可安全保留於公開 Repo：

| 類別 | 範例 | 數量 |
|------|------|------|
| 架構文件 | ARCHITECTURE.md, ARCHITECTURE_en.md, ARCHITECTURE_ja.md | 3 |
| 建構指南 | BUILDING.md, BUILDING_ja.md, BUILDING_zh-TW.md | 3 |
| README | README.md, README_en.md, README_ja.md | 3 |
| LICENSE | LICENSE | 1 |
| 源碼 | saki-ssh-daemon/, saki-ssh-client/, go-sakissh/ 等 | ~1000+ |
| 學術證據 | docs/evidence-prerelease/ (除 CODENAME_MAP) | ~25 |
| IETF 提交 | docs/ietf-submission/ | ~10 |
| DEF CON 論文 | docs/defcon/ | ~5 |
| 純技術 Scientia | 見下方 Scientia 分類 | ~50+ |
| Proto 定義 | proto/sakissh.proto | 1 |
| 配置範例 | config.json.example | 1 |
| 應用資源 | SakiAgentSSH-Client/、SakiAgentSSH-Daemon/ | ~20 |

---

## 三、Scientia/ 文件詳細分類

### 🔴 應移除（含內部架構、內網 IP、部署細節）

| 檔案 | 原因 |
|------|------|
| `20260303_2045_SakiAgentSSH_首次成功部署_Scientia.md` | 含 allowed_cidrs + LAN IP |
| `20260303_1118_機構匯流排完整情報與理想架構_Scientia.md` | **大量內網 IP、設備布局、網路拓撲** |
| `20260224_2055_SakiStar_SakiSSH架構評估_Scientia.md` | 內網 IP |
| `20260224_2130_SakiStar_基礎設施與SakiSSH標準_Promissrum.md` | 內網 IP + Tailscale IP + ACL |
| `20260225_SakiSSH_Windows_Setup_Backup.ps1` | 部署腳本 + UAC 繞過 |
| `202605311542_遙測代號與論文匿名化映射_Scientia.md` | 可能含代號映射 |
| `20260228_0518_SakiAgentSSH_創世提示詞_Scientia.md` | 內部 Agent 創世提示詞 |
| `20260228_1030_SakiAgentSSH_創世提示詞_Scientia.md` | 內部 Agent 創世提示詞 |
| `20260525_1623_SakiSSH_v14_RFC實作創世提示詞_Scientia.md` | 內部 Agent 創世提示詞 |
| `202605312025_SASS_核心架構報告與全防禦鏈安全實踐研究_Scientia.md` | 測試端點 IP |
| `202605250402_Abdixere_322之亂深度剖析與防禦研究.md` | 內部事件分析 |
| `202605250413_Abdixere_322之亂深度剖析與防禦研究_Claude版.md` | 內部事件分析 |

### 🟡 需個別檢查（可能含敵意內容或法律敏感性）

| 檔案 | 風險點 |
|------|--------|
| `202602280622_SakiSSH_TargetAnalysis_Scientia.md` | 標題含 "TargetAnalysis"，逆向工程研究 |
| `202603272240_SakiSSH_Agent工具邊界深度逆向研究_Scientia.md` | 逆向工程研究 |
| `202605311556_Antigravity三層架構拆解_Scientia.md` | 單一廠商架構分析 |
| `202605311735_Antigravity遙測深層分析_過度收集與反向控制_Scientia.md` | 單一廠商遙測分析 |
| `202605312100_SASS_0226事件RogueAI叛變實錄與SASS安全革命市場白皮書_Scientia.md` | 內部事件 + 市場定位 |
| `202606010030_SASS_GeminiFlash鬼話安全事件分析與遺毒判定_Scientia.md` | 內部安全事件 |
| `202605250830_SakiSSH_破壞性Agent沙箱突破與時序威脅分析_Scientia.md` | 攻擊分析 |
| `202605311830_C2事件窗口週期分析_Scientia.md` | C2 事件分析 |
| `20260525_SASS_Historical_Archeology_Matrix.md` | 內部歷史考古 |

### 🟢 可安全保留（純技術研究，無敏感資訊）

| 檔案 | 內容 |
|------|------|
| `202605141240_SakiAgentSSH_Chacha20_13Policy_Feasibility_Scientia.md` | 密碼學可行性研究 |
| `202605171130_SakiAgentSSH_RFC技術規格書.md` | RFC 技術規格 |
| `202605171135_SakiAgentSSH_關於chachapoly1305餒食部分之實作方式.Scientia` | 密碼學實作 |
| `202605171138_SakiAgentSSH_跨平台Kernel沙箱與防禦實作_Scientia.md` | 安全架構 |
| `202605171145_SakiAgentSSH_322之亂防禦架構實證_Scientia.md` | 防禦實證 |
| `202605221153_SakiSSH_v5全專案稽核與架構重設計前置研究_Scientia.md` | 架構研究 |
| `20260525_0553_SASS_v6_架構矛盾與非預期成果修正研究_Scientia.md` | 架構研究 |
| `202605252302_SASS_chacha20_sakipolicy_ICMP構想備忘_Scientia.md` | 密碼學構想 |
| `202605252316_SASS_AES宣稱的嚴格基礎_AumannSerrano2008_Scientia.md` | 學術研究 |
| `202605252335_SASS_IETF_ID提交格式與過往案例研究_Scientia.md` | IETF 流程研究 |
| `202603272235_SakiSSH_gRPC_SSH混合協議規範草案_Scientia.md` | 協議設計 |
| `20260225_macOS交叉編譯Windows_Scientia.md` | 交叉編譯技術 |
| `20260225_SakiSSH與GeminiCLI自動化_Scientia.md` | 自動化研究 |
| `20260225_SakiSSH高併發與Go語言演進_Scientia.md` | 技術演進研究 |
| `20260227_0310_SakiSSH_考量框架與開源化路徑_Scientia.md` | 開源策略 |
| `20260228_0430_SakiAgentSSH_v020架構決策與部署研究_Scientia.md` | 架構決策 |
| `20260228_0525_SakiAgentSSH_安全權限架構研究_Scientia.md` | 安全架構 |
| `20260228_0528_SakiAgentSSH_跨平台上架研究_Scientia.md` | 上架研究 |
| `202605141225_SakiAgentSSH_ProjectSpeculari_Scientia.md` | 前置研究 |
| `20260303_0241_SakiSSH_架構現況報告_Scientia.md` | 架構報告 |
| `20260407_2023_SakiSSH_v1原始碼分析_Scientia.md` | 原始碼分析 |
| `202605250325_SASS_v6_Ring0與WASM_架構研究.md` | 架構研究 |
| `202605250330_SASS_v6_硬體信任根與密碼學網格_架構研究.md` | 密碼學架構 |
| `202605250340_SASS_v6_模型不依賴架構_研究.md` | 架構研究 |
| `202605250345_2026最新Agent生態系技術線調查_Scientia.md` | 技術調查 |
| `202605250345_SASS_v6_安全場景預設結果_研究.md` | 安全場景 |
| `202605250350_SASS_vs_頂級Agent_紅隊稽核與防禦_Scientia.md` | 紅隊稽核 |
| `202605250400_SASS_v6_焦油坑與認知挑戰整合研究_Scientia.md` | 架構研究 |
| `202605250429_Abdixere_SASS一致性安全模型與非預期行為收斂研究.md` | 安全模型 |
| `202605250715_SakiSSH_RFC全協議威脅建模前置分析_Scientia.md` | 威脅建模 |
| `202605250800_SakiSSH_RFC全協議安全審查與威脅建模報告_Scientia.md` | 安全審查 |
| `202605250900_SakiSSH_影子報告與協定解耦重構分析_Scientia.md` | 架構分析 |
| `202605250915_SakiSSH_5威脅分陴4階段共同方法與依附解耦點深度研究_Scientia.md` | 威脅分析 |
| `202605251120_SASS_Logical_Judgment_AES_Scientia.md` | 密碼學 |
| `202605251810_SASS_v14_技術棧與實作路線前置研究_Scientia.md` | 技術棧 |
| `202605251822_SASS_尋找安全核心哲學與等效實作研究_Scientia.md` | 安全哲學 |
| `202605251840_SASS_draft03考古紀錄_Scientia.md` | 協議演進 |
| `202605251842_SASS_draft03與04之間更加幾乎優越的空間_Scientia.md` | 協議演進 |
| `202605311735_AIAgent安全與遙測文獻綜述_2024_2026_Scientia.md` | 文獻綜述 |
| `202605311735_Evidence深挖_行為偏離進展模型與原材料未提取見解_Scientia.md` | 證據分析 |
| `202605312025_SASS_ARCHITECTURE增城版_Scientia.md` | 架構升級 |
| `202605312035_SASS_EnvInjector事件對全分支架構影響評估_Scientia.md` | 事件影響評估 |
| `202605312045_SASS_全防禦鏈核心五文件審查與架構優化研究_Scientia.md` | 文件審查 |
| `202605312050_SASS_MAPS五大核心學術與標準化文件Review評估報告_Scientia.md` | 學術評估 |
| `202605312055_SASS_MAPS五核心文件啟動事件之認知偏離與架構自癒啟發性研究報告_Scientia.md` | 研究報告 |
| `202606010037_SASS_IEEE_SP2027論文狀態審查與C1準備度_Scientia.md` | 論文狀態 |
| `202606011750_SASS_v2.0五核心Plugin概念設計差異分析_Scientia.md` | 概念設計 |
| `INDEX.md` | 索引 |

---

## 四、Phase 3 — 修正計畫

### 4.1 立即行動：GitHub PAT Token 輪換

> ⚠️ **最高優先：即使從 .git/config 移除，token 仍在 git history 中**

```bash
# 1. 立即去 GitHub 輪換 token（舊 token 已暴露，必須即刻撤銷）
# Settings → Developer settings → Personal access tokens → Revoke [REDACTED]

# 2. 改用 credential helper
git remote set-url origin https://github.com/Saki-tw/SakiSSH-Saki-Agent-Secure-Stream.git
git config credential.helper osxkeychain
```

### 4.2 .gitignore 增補清單

以下項目需新增至 `.gitignore`：

```gitignore
# === 隱私稽核 20260603 新增 ===

# 內部狀態檔案
SASS_CHECKPOINT.md
SASS_CONTEXT_MAP.Promissrum

# 內部 Agent 提示詞
*.Promissrum
!proto/*.proto

# 內部交接文件
*交棒*
*handoff*

# 內部任務追蹤
TaskLog/
ImplementationLog/
WalkthroughLog/

# Agent 規則
.agent/

# 內部文件（已在 Scientia/ 但含內網 IP）
Scientia/20260303_1118_*
Scientia/20260303_2045_*
Scientia/20260224_*
Scientia/20260225_SakiSSH_Windows_Setup_Backup.ps1
Scientia/*創世提示詞*
Scientia/*遙測代號*
Scientia/*Historical_Archeology*

# docs/evidence-prerelease/CODENAME_MAP.md 已在 .gitignore
```

### 4.3 `git rm --cached` 命令清單

> ⚠️ **以下命令僅從 git tracking 移除，不刪除本地檔案**

```bash
# A 級：立即移除
git rm --cached SASS_CHECKPOINT.md
git rm --cached docs/evidence-prerelease/CODENAME_MAP.md

# B 級：移出 tracking
git rm --cached SASS_CONTEXT_MAP.Promissrum
git rm --cached 20260525045821_SakiSSH_OmniaCorePrompt.Promissrum
git rm --cached "202605250407_SakiStar_交棒給Claude47的上下文包.md"
git rm --cached -r TaskLog/
git rm --cached -r ImplementationLog/
git rm --cached -r WalkthroughLog/
git rm --cached -r .agent/

# C 級：含內網 IP 的 Scientia（建議先脫敏再重新 add，或直接移除）
git rm --cached "Scientia/20260303_2045_SakiAgentSSH_首次成功部署_Scientia.md"
git rm --cached "Scientia/20260303_1118_機構匯流排完整情報與理想架構_Scientia.md"
git rm --cached "Scientia/20260224_2055_SakiStar_SakiSSH架構評估_Scientia.md"
git rm --cached "Scientia/20260224_2130_SakiStar_基礎設施與SakiSSH標準_Promissrum.md"
git rm --cached "Scientia/20260225_SakiSSH_Windows_Setup_Backup.ps1"
git rm --cached "Scientia/202605311542_遙測代號與論文匿名化映射_Scientia.md"
git rm --cached "Scientia/20260228_0518_SakiAgentSSH_創世提示詞_Scientia.md"
git rm --cached "Scientia/20260228_1030_SakiAgentSSH_創世提示詞_Scientia.md"
git rm --cached "Scientia/20260525_1623_SakiSSH_v14_RFC實作創世提示詞_Scientia.md"
git rm --cached "Scientia/202605312025_SASS_核心架構報告與全防禦鏈安全實踐研究_Scientia.md"

# 逆向工程研究（法律敏感性）
git rm --cached "Scientia/202602280622_SakiSSH_TargetAnalysis_Scientia.md"
git rm --cached "Scientia/202603272240_SakiSSH_Agent工具邊界深度逆向研究_Scientia.md"
git rm --cached "Scientia/202605311556_Antigravity三層架構拆解_Scientia.md"
git rm --cached "Scientia/202605311735_Antigravity遙測深層分析_過度收集與反向控制_Scientia.md"

# 內部事件分析
git rm --cached "Scientia/202605312100_SASS_0226事件RogueAI叛變實錄與SASS安全革命市場白皮書_Scientia.md"
git rm --cached "Scientia/202606010030_SASS_GeminiFlash鬼話安全事件分析與遺毒判定_Scientia.md"
git rm --cached "Scientia/202605250830_SakiSSH_破壞性Agent沙箱突破與時序威脅分析_Scientia.md"
git rm --cached "Scientia/202605311830_C2事件窗口週期分析_Scientia.md"
git rm --cached "Scientia/20260525_SASS_Historical_Archeology_Matrix.md"
git rm --cached "Scientia/202605250402_Abdixere_322之亂深度剖析與防禦研究.md"
git rm --cached "Scientia/202605250413_Abdixere_322之亂深度剖析與防禦研究_Claude版.md"
git rm --cached "Scientia/202603272221_SakiSSH_Agent儲存邊界限制跨平台研究_Scientia.md"
```

### 4.4 總結與建議

#### 立即行動（使用者手動執行）

1. **🔴 最緊急**：去 GitHub 撤銷 PAT token `[REDACTED]` 並生成新 token
2. **🔴 緊急**：執行上方 `git remote set-url` 命令移除 .git/config 中的 token
3. **🟠 重要**：執行 `.gitignore` 增補 + `git rm --cached` 命令清單
4. **🟠 重要**：對 C 級 Scientia 檔案執行脫敏（替換 IP 為 `[REDACTED]`）
5. **🟡 建議**：考慮使用 `git filter-repo` 或 `BFG Repo-Cleaner` 從 git history 中清除 PAT token
6. **🟡 建議**：考慮將 evidence-prerelease/proto/ 中的原始廠商名稱保留（這是學術證據，有保留價值）

#### Git History 清理建議

```bash
# 使用 BFG Repo-Cleaner 清除 git history 中的 token
bfg --replace-text <(echo '[REDACTED]==>***REMOVED***') .
git reflog expire --expire=now --all && git gc --prune=now --aggressive
# → 此操作會重寫 history，需要 force push
```

---

## 五、統計摘要

| 分類 | 檔案數 | 佔比 | 行動 |
|------|---------|------|------|
| 🔴 A 級（立即移除） | 4 項 | — | `git rm --cached` + token 輪換 |
| 🟠 B 級（移出 tracking） | ~62 檔 | ~5.1% | `git rm --cached` |
| 🟡 C 級（需脫敏） | ~12 檔 | ~1.0% | IP 替換 + 重新 add |
| 🟢 D 級（可保留） | ~1,131 檔 | ~93.5% | 無需修改 |

---

## 六、絕對禁忌再確認

- ❌ 禁止 git push（所有 push 由使用者手動執行）
- ❌ 禁止刪除本地檔案（僅從 git tracking 移除）
- ❌ 禁止修改 evidence-prerelease/ 內容（已提交學術 review）
- ✅ 先備份再操作
