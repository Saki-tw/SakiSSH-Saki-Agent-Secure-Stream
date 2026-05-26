# SASS v1.4 TaskMELIUS 01 — Phase 0 死碼整合

> **建立時間**：2026-05-25 18:20 (UTC+8)
> **對應 task.md**：Phase 0 死碼整合與依賴修復
> **展開比率**：1:5

---

## 任務第一步：Cargo.toml 依賴修復

1. 確認現有 Cargo.toml 中的 13 個依賴
2. 分析 17 個死碼模組的 use 聲明，列出所有缺失 crate
3. 確認每個 crate 的最新穩定版本
4. 寫入 Cargo.toml 新增依賴
5. 執行 `cargo check` 確認依賴解析正常 → 立即開始任務第二步

## 任務第二步：mod 宣告整合

1. 讀取現有 main.rs 的 `mod config;`
2. 新增 17 個 mod 宣告
3. 確認模組間依賴關係（哪些模組引用了其他模組）
4. 排序 mod 宣告以避免循環依賴
5. 執行 `cargo check` 查看錯誤 → 立即開始任務第三步

## 任務第三步：編譯錯誤修復（預估最耗時）

1. 收集 cargo check 錯誤訊息
2. 分類錯誤（型別不匹配、缺失 import、API 變更、不存在欄位）
3. 逐一修復，優先修復無依賴的模組（codec, policy, env_injector）
4. 再修復有依賴的模組（v6_integration 使用了 proto 不存在的欄位——需要 stub 或重寫）
5. 執行 `cargo clean && cargo check` 全量通過 → 立即開始任務第四步

## 任務第四步：驗證與歸檔

1. `cargo test`（如有現有測試）
2. 確認所有 17 個模組的 pub 介面可被 main.rs 存取
3. 更新 Scientia 記錄 Phase 0 完成結果
4. 更新 task.md 標記 Phase 0 完成
5. 開始 Phase 1（mTLS）的第一步