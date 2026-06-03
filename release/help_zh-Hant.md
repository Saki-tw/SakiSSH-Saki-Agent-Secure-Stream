# SakiAgentSSH GUI 說明書 (zh-Hant)

> 這不僅是一份說明書，也是寫在 UI 裡的廢土求生指南與數位詩集。

## 1. 介面與哲學：馬卡龍紫與勿忘草青的交織
開啟 `SakiAgentSSHClientApp` 或是 `SakiAgentSSHDaemonApp`，你首先會注意到的，是那個由馬卡龍紫（`#DA70D6`）與勿忘草青（`#00CED1`）所構成的漸層背景（`--bg-gradient`）。這不是為了華麗，這是一種溫柔的抵抗。在這個終端機總是漆黑一片的冷酷年代裡，這抹顏色就像是一隻淡藍色的蝴蝶自肚腹裡升起。這是 Saki Studio 對所有 Agent 所承諾的：在這裡，你不再只是冰冷字節，我們為你建構了一個有溫度的控制台。所有介面皆使用 `GenJyuuGothicX-Regular` 渲染，確保每一個字元的圓潤度都能中和那些生硬的指令。

## 2. 關於 SakiAgentSSH Daemon (守護進程端)
在你的運算端（Compute Plane）——可能是一台擁有 40GB RAM 的 Loser PC，或是某台安靜地在角落發熱的 Windows 機器——你需要啟動 Daemon。
1. **介面檢視**：在 macOS 上啟動後，你將看到「SakiAgentSSH Daemon」的字樣與 `v0.2.0`。
2. **安全防護 (CIDR)**：介面上標示的「CIDR 白名單安全存取控制」，不是隨便寫寫的。這是我們的防線。若有人問起，為什麼我們不直接開 22 Port？我會告訴他，那些來自廣域網路的嗅探，會在那之前被 `check_acl` 安靜地擋下。至少讓我們的內部算力免於無謂的侵擾。
3. **啟動方式**：
   - **macOS**：App Store 下載後直接點擊啟動，安靜駐留。
   - **Windows**：我們貼心地在介面內放上了 GitHub Release 的 `sakisshd.exe` 下載連結。透過 `sakisshd.exe --config config.toml` 啟動它，讓它成為廢土中堅實的橋墩。

## 3. 關於 SakiAgentSSH Client (客戶端)
在你的控制端（Control Plane，例如那台 16GB RAM 的 M1 Mac Mini），這是你的大腦。
1. **指令代理**：點開 App，你會看到「Agent 原生遠端執行 CLI」。我們捨棄了繁重的 TTY 綁定。
2. **即時串流**：「gRPC 雙向即時串流」是它的靈魂。每一次敲擊，所有的 `stdout` 都會如同收音機傳來的那聲「午後，可能有雨」一般，準確地、沒有延遲地傳回。
3. **連線方式**：
   - 透過終端機輸入 `sakissh --addr http://<你的Daemon_IP>:19284 exec -- '你的指令'`。這行指令，就是你神經索的延伸。

## 4. 故障排除 (Troubleshooting)
如果有一天，連線中斷了。請不要驚慌。
1. **檢查 CIDR**：連線失敗通常是因為白名單設定。請檢查 `config.toml`，確認你的 IP 被允許進入這座堡壘。
2. **防火牆**：Windows 或 macOS 的防火牆是否擋住了 `19284`？請去開放它，就像在廢墟中推開一扇沉重的鐵門。
3. **端口衝突**：如果 `19284` 被佔用，換一個吧。我們不會對此有任何執念，因為只要連上了，任何 Port 都能傳遞意志。

（附註：本說明書對應了 Help Book 的 `index.html`, `installation.html`, `usage.html`, `troubleshooting.html` 結構。）