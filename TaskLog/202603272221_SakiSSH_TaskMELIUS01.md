# SakiAgentSSH 協議研究 TaskMELIUS 01：Agent 儲存空間邊界限制機制研究

> 建立時間：2026-03-27 22:21 (UTC+8)
> 對應 task.md Phase 1.4

## 任務第一步：Gemini CLI 的工具邊界與沙箱機制

1. 確認 Gemini CLI 的安裝位置與配置結構
2. 分析 Gemini CLI 的工具執行權限模型（allowedTools/blockedTools）
3. 研究 Gemini CLI 的檔案系統存取限制
4. 確認 Gemini CLI 是否有 chroot/namespace 隔離
5. 確認下一步：進入 Antigravity 研究

## 任務第二步：Antigravity (Windsurf) 的工具邊界與沙箱機制

1. 確認 Antigravity 的 CortexStepType 與工具分類
2. 分析 Antigravity 的 LanguageServer 工具映射
3. 研究 Antigravity 的檔案系統存取限制
4. 確認 Antigravity 的 token/權限隔離機制
5. 確認下一步：進入 Claude Code 研究

## 任務第三步：Claude Code 的工具邊界與沙箱機制

1. 確認 Claude Code 的安裝位置與配置結構
2. 分析 Claude Code 的權限模型（Allowed/Denied Tools）
3. 研究 Claude Code 的檔案系統存取限制
4. 確認 Claude Code 的沙箱/容器化機制
5. 確認下一步：建立統一能力矩陣

## 任務第四步：建立 Agent 統一能力矩陣

1. 彙整所有 Agent 的工具執行能力對照
2. 識別各 Agent 的儲存空間邊界弱點
3. 定義本協議需要覆蓋的限制維度
4. 建立方法論有效性評估框架
5. 產出 Scientia：Agent 儲存邊界限制研究

## 任務第五步：設計協議層的邊界限制方案

1. 設計 gRPC 層的路徑限制 interceptor
2. 設計 SSH 風格的 channel+subsystem 權限模型
3. 設計 daemon 側的強制邊界（chroot-like）
4. 撰寫協議規範草案
5. 確認下一步：進入 Phase 2 協議架構設計
