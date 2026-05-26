# TaskMELIUS5: 生成 SakiAgentSSH 三語系標準 Github 文件

**目標**：
針對 SakiAgentSSH 專案內的標準 Github 文件，如果只有一個語系版本，自動以三語系語境（台北詩人、東京少女、波士頓科學家）生成對應版本，並在原檔案加入多語系導覽連結。

**目前盤點結果**：
1. `README.md` (有混合語系字句，但無獨立的 `README_ja.md` / `README_en.md`)
2. `ARCHITECTURE.md` (目前只有中文版)
3. `BUILDING.md` (目前為英文版)

**執行計畫**：
1. **處理 `README.md`**：
   - 建立 `README_ja.md`（東京少女語氣）。
   - 建立 `README_en.md`（波士頓科學家語氣）。
   - 修改原 `README.md`，在頂部加入語言切換連結：`[🇹🇼 繁體中文](README.md) | [🇯🇵 日本語](README_ja.md) | [🇺🇸 English](README_en.md)`。
2. **處理 `ARCHITECTURE.md`**：
   - 原版保留為中文（或重新命名為 `ARCHITECTURE_zh-TW.md` 並更新，維持 `ARCHITECTURE.md` 為入口）。
   - 建立 `ARCHITECTURE_ja.md` 和 `ARCHITECTURE_en.md`。
   - 加入語言切換連結。
3. **處理 `BUILDING.md`**：
   - 建立 `BUILDING_zh-TW.md` 和 `BUILDING_ja.md`。
   - 加入語言切換連結。
