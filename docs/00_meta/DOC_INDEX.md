# SakiAgentSSH 文檔索引 (DOC_INDEX)

> **更新時間**：2026-03-06
> **專案版本**：v1.0.0（架構報告 v2.0）

## 專案概述

SakiAgentSSH — 基於 gRPC 的跨機 Agent 執行橋樑，取代傳統 SSH，專為高頻次、非互動式 Agent 自動化設計。

## 核心文檔

| 文檔 | 說明 | 狀態 |
|------|------|------|
| [README.md](../../README.md) | 專案入口 | ✅ |
| [ARCHITECTURE.md](../../ARCHITECTURE.md) | 架構現況報告 v2.0 | ✅ |
| [BUILDING.md](../../BUILDING.md) | 編譯指南 (EN) | ✅ |
| [BUILDING_zh-TW.md](../../BUILDING_zh-TW.md) | 編譯指南 (繁中) | ✅ |
| [BUILDING_ja.md](../../BUILDING_ja.md) | 編譯指南 (日文) | ✅ |

## 多語系文檔

| 語系 | README | ARCHITECTURE | BUILDING |
|------|--------|-------------|----------|
| 🇹🇼 繁中 | ✅ README.md | ✅ ARCHITECTURE.md | ✅ BUILDING_zh-TW.md |
| 🇺🇸 EN | ✅ README_en.md | ✅ ARCHITECTURE_en.md | ✅ BUILDING.md |
| 🇯🇵 JA | ✅ README_ja.md | ✅ ARCHITECTURE_ja.md | ✅ BUILDING_ja.md |

## 知識庫 (Scientia/)

| 文件 | 主題 | 日期 |
|------|------|------|
| 20260224_2055_SakiStar_SakiSSH架構評估 | 初始架構評估（從 SakiStar 同步） | 2026-02-24 |
| 20260224_2130_SakiStar_基礎設施與SakiSSH標準 | 基礎設施標準（從 SakiStar 同步） | 2026-02-24 |
| 20260225_macOS交叉編譯Windows | macOS→Windows 交叉編譯 | 2026-02-25 |
| 20260225_SakiSSH與GeminiCLI自動化 | Gemini CLI 自動化整合 | 2026-02-25 |
| 20260225_SakiSSH高併發與Go語言演進 | 高併發設計決策 | 2026-02-25 |
| 20260225_SakiSSH_Windows_Setup_Backup.ps1 | Windows 部署腳本 | 2026-02-25 |
| 20260227_0310_SakiSSH_考量框架與開源化路徑 | 開源化路徑評估 | 2026-02-27 |
| 20260228_0430_SakiAgentSSH_v020架構決策與部署研究 | v0.2 架構決策 | 2026-02-28 |
| 20260228_0518_SakiAgentSSH_創世提示詞 | 創世提示詞 | 2026-02-28 |
| 20260228_0525_SakiAgentSSH_安全權限架構研究 | 安全權限設計 | 2026-02-28 |
| 20260228_0528_SakiAgentSSH_跨平台上架研究 | 跨平台上架策略 | 2026-02-28 |
| 20260228_1030_SakiAgentSSH_創世提示詞 | 創世提示詞（更新版） | 2026-02-28 |
| 202602280622_SakiSSH_TargetAnalysis | 目標分析 | 2026-02-28 |
| 20260303_0241_SakiSSH_架構現況報告 | 架構報告 v1.0 快照 | 2026-03-03 |
| 20260303_1118_機構匯流排完整情報與理想架構 | 匯流排架構（從 SakiStar 同步） | 2026-03-03 |
| 20260303_2045_SakiAgentSSH_首次成功部署 | Windows Service 首次部署 | 2026-03-03 |

## Release 資源

| 通路 | 路徑 | 狀態 |
|------|------|------|
| Homebrew Cask | `release/homebrew-cask/` | ✅ |
| Winget | `release/winget/` | ✅ |
| Scoop | `release/scoop/` | ✅ |
| App Store Review | `release/REVIEW_GUIDE_*.md` | ✅ |

## 稽核備註 (2026-03-06)

### 散落文檔

WalkthroughLog/ 中有 4 份 SakiWeb 相關文檔不屬於本專案：
- `202602280714_SakiWeb_Deployment_Walkthrough.md`
- `202602281015_SakiWeb_Projects_Walkthrough.md`
- `202602281022_SakiWeb_LegalDocs_Walkthrough.md`
- `202602281027_SakiWeb_LanguageLinks_Walkthrough.md`

> 建議：未來稽核時移至 SakiWeb 專案。

### 品質風險

- ⚠️ **自動化測試 = 0** — 建議至少覆蓋 ACL 與 config 解析
