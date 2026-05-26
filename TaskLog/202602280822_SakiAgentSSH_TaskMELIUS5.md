# TaskMELIUS5: 整合 SakiAgentSSH 至 SakiWeb 入口

**目標**：
將 SakiAgentSSH 加入至 `SakiWeb/content/projects.md` 中的「主要專案」清單，並使用對應的三語境風格描述，以便訪客可以從入口導航至 `/saki-agent-ssh/` (或 `/SakiAgentSSH/`) 頁面。

**執行步驟**：
1. 觀察 `projects.md` 中的 SakiClip 區塊，了解 HTML 結構與 `glass-card` 的設計模式。
2. 撰寫一段符合台北硬核詩人語氣的 SakiAgentSSH 簡介，長度與 SakiClip 相當。
3. 將這段介紹插入到 `projects.md` 的 SakiStar 區塊附近，因為兩者都是通訊層相關。
4. 使用 `replace` 修改 `projects.md`。
5. （同步檢查是否需要編譯與部屬，以確認變更生效）。