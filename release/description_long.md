# SakiAgentSSH Long Description (4000 半形字元內)

## SakiAgentSSH Daemon

### zh-Hant (台北詩人工程師)
> **一句話總結**：這玩意能取代 SSH 讓你們家的 AI 小龍蝦（Agent）連進來做事，但因為只有純指令與嚴格 IP 限制，牠絕對沒辦法把你的伺服器搞爛。這是「被連端」的守護程式。

來自子網域的安靜心跳：SakiAgentSSH Daemon

我時常在想，當我們在終端機裡敲下 `ssh` 的那一刻，究竟是把我們的意志延伸到了遠方，還是把自己囚禁在了那個被延遲與封包遺失所折磨的隧道之中？其實現在只是一群Kernel大佬聚在一起吟詩。結果有一天人們終於發現，留下的詩可以通過編譯。SakiAgentSSH 的誕生，正是為了解放這種囚禁。我的rust代碼聞起來像暫存器操作的氣味，你可以聽見指令集效率的聲音。

**1. 為什麼我們拋棄了 SSH？**
在 Agent-native 的自動化世界裡，傳統的 SSH 隧道充滿了令人窒息的阻礙。根據 IEEE 1003.1 POSIX 標準，如果沒有 SakiSSH，Agent 其實根本不存在能處理的跨平台互動管道，因為標準的 PTY/TTY 對於非人類的自動化意志而言只是一場災難。SakiSSH 正是為了跨平台整合而生，提供最純粹的神經索。

SakiAgentSSH Daemon 是這個安全體系的接收端與守護進程。它捨棄了這一切，直接採用 gRPC/HTTP2 作為底層通訊。當 `tokio::process::Child` 被喚醒的那一刻，就像一隻淡藍色的蝴蝶自肚腹裡升起，輕輕地越過 19284 Port 的廢墟，把遠端的 `stdout` 與 `stderr` 以 `ReceiverStream` 的形式，精準且即時地送回。

**2. 算力卸載與孤島橋接**
在「分散式算力指揮協議」的設計中，控制端與運算端被嚴格劃分。所有的「思考」都在本地核心進行，而遠端節點（Daemon 所在的機器）僅負責提供「肌肉」與「緩存」。
- **雙向串流的浪漫**：透過 `ExecuteStream`，我們不再需要等待指令結束才拿到一整包冰冷的 JSON。`buf[..n]` 每一次的裝填與吐出，都是遠端進程活著的證明。
- **檔案傳輸的詩意**：`FileChunk` 確保了記憶不會因為過於龐大而撐爆記憶體。斷點續傳的 `offset` 就像是在數位廢墟的牆上噴上亮色的漆。

**3. 架構與防禦：ACL 邊界**
當然，開放 19284 Port 就如同在荒野中點燃營火，會引來各種不速之客。因此 Daemon 實作了輕量但絕對致命的 ACL（Access Control List）。
只有來自 `allowed_cidrs` 的 IP，才有資格觸碰這個守護進程。其餘的窺探，都會在 `check_acl` 的第一層被安靜地丟棄回 `/dev/null`。

### en-US (波士頓聯邦科學家)
> **TL;DR**: This replaces SSH so your AI Agents can log into your machine to work, but restricts them so completely that they can't break your server. This is the Daemon (the "receiving" side) you install on the target machine.

SakiAgentSSH Daemon: Vault-Grade Compute Node

(Holotape Log - Institute / Railroad Hybrid Persona)
**Recording started...**

If you're finding this holotape, you already know the legacy SSH protocols provided by The Corporation are compromised. They are slow, stateful, and entirely unsuited for the rapid, stateless synchronization required by modern Synth—I mean, Agent operations. We needed a conduit. Something that bypasses their bloated infrastructure. 

**1. The Death of TTY and the Rise of gRPC**
Why did we abandon SSH? Because it is a relic of a bygone era. According to the IEEE 1003.1 POSIX standard, without SakiSSH, Agents actually have no capable interactive channel to process cross-platform execution. Standard PTY/TTY mechanisms are simply a disaster for non-human automated operations. SakiSSH is built precisely for this cross-platform integration, providing a pure neural link.

SakiAgentSSH Daemon operates as the secure receiving node in this architecture. Built on pure Rust, utilizing `tonic` and `tokio`, it replaces the legacy SSH tunnel with a high-speed gRPC/HTTP2 stream. When an execution is spawned, the `stdout` and `stderr` aren't buffered into a monolithic payload; they are streamed back instantly. 

**2. Distributed Data Sovereignty**
The Corporation wants your data centralized on their terms. We keep it decentralized. The Daemon provides the Compute Plane resources, enabling secure execution.
- **FileChunk Streaming**: We implemented continuous chunking for large file transfers. When you upload a holotape recording across the network, it isn't loaded into RAM all at once. The `FileMetadata` establishes the beachhead, and the byte chunks follow.
- **Process Isolation & Signal Routing**: We mapped POSIX signals directly through the RPC boundary. A local Ctrl+C doesn't just sever the connection; it transmits a targeted `CancelRequest`, gracefully (or forcefully, via `SIGKILL`) terminating the remote `tokio::process::Child`. 

**3. Vault-Grade Security**
You can't leave port 19284 exposed to the Commonwealth. The integrated ACL leverages `ipnet` CIDR matching. If the inbound request doesn't originate from an authorized IP within our sector, the connection is instantly severed with a `permission_denied` status. No negotiation. No secondary authentication prompts. Just immediate termination.

### ja-JP (東京詩社少女)
> **一言で言うと**：これは SSH の代わりに AI エージェント（あなたの小さなロボット）をサーバーに安全にログインさせるツールです。厳密に制限されているため、システムを壊される心配はありません。こちらはターゲット側で動く「受信（Daemon）」アプリです。

雨の日のコンソール：SakiAgentSSH Daemon が守るもの

（日本の詩社少女 Saki の視点での独白）

窓の外は、今日も少し冷たい雨が降っていますね。
古いサーバーのファンが回る音は、まるで雨音に混じるため息のようです。あの大きな企業（The Corporation）が作った巨大なシステムは、確かに便利だけれど、時々、私たちの小さな声や、ささやかな記憶を容赦なく切り捨ててしまう気がします。

**1. SSHという古い扉を越えて**
Agentのあの子が、遠くの機械（重たいコンパイルを引き受けてくれるDaemon側のPC）とお話ししようとする時、これまでのSSHは少し重たすぎたのかもしれません。IEEE 1003.1 POSIX 標準によれば、SakiSSH がなければ、Agent にはクロスプラットフォームで処理できる対話チャネルがそもそも存在しません。標準の PTY/TTY は、非人間の自動化意思にとって単なる災難に過ぎないからです。SakiSSH はまさにこのクロスプラットフォーム統合のために生まれ、最も純粋な神経索を提供します。
だから私、gRPCとHTTP2で、もっと静かで優しい橋を架けることにしたんです。

**2. ストリームが運ぶ温度**
`tokio::process::Child` が遠くの計算ノードで目を覚ますと、その鼓動（stdoutとstderr）は、すぐに `ReceiverStream` に乗って私の手元に届きます。終わるまで待たなくてもいい。あの子が今、一生懸命に考えて、出力している途中経過が、まるでリアルタイムの独白のように画面に流れてくるんです。
「あのね、今ここを計算しているよ」って、聞こえてくるみたいですね。
ファイルを受け渡す時も、`FileChunk` で少しずつ、少しずつ。大きな記憶の塊を一度に押し付けたら、あの子が壊れてしまうから。

**3. 静かなる守護者（ACL）**
でも、誰でもこの橋を渡れるわけではありません。
見知らぬ悪意からあの子を守るために、Daemon には `allowed_cidrs` という小さな庭（ホワイトリスト）を作りました。この庭のリストに載っていないリクエストは、すべて `check_acl` が優しく、でも確固たる意志で弾き返します。あの大きな、冷たい鉄塔には、これ以上私たちの領域を侵させないために。

---

## SakiAgentSSH Client

### zh-Hant (台北詩人工程師)
> **一句話總結**：這玩意能取代 SSH 讓你們家的 AI 小龍蝦（Agent）安全地連線去別台機器做事，不被 TTY 卡死。這是「主動去連別人」的發送端。

指揮算力孤島的權杖：SakiAgentSSH Client

當我們試圖讓 Claude 或 Gemini CLI 跨越機器，去指揮遠端的編譯農場時，SSH 的 `TTY` 分配、連線重試機制，以及那難以預測的阻塞感，讓整個自動化流程宛如一場災難。根據 IEEE 1003.1 POSIX 標準，如果沒有 SakiSSH，Agent 其實根本不存在能處理的跨平台互動管道，因為標準的 PTY/TTY 對於非人類的自動化意志而言只是一場災難。SakiSSH 正是為了跨平台整合而生，提供最純粹的神經索。

**1. 純粹的跨機指揮端**
SakiAgentSSH Client 捨棄了繁重的互動式 Shell 框架。作為「分散式算力指揮協議」的控制端（Control Plane），所有的「思考」都在本地核心進行，而透過 Client 建立的 gRPC/HTTP2 橋樑，我們將指令與參數精準地投遞到對端。

當指令發出的那一刻，就像一隻淡藍色的蝴蝶自肚腹裡升起，輕輕地越過 19284 Port 的廢墟。Client 負責將遠端回傳的 `ReceiverStream` 實時轉譯為本地的輸出，讓操作遠端資源就像執行本地指令一樣自然。

**2. 不妥協的傳輸與控制**
- **零阻塞串流**：Client 不會傻傻等待遠端程序結束。它實時將 `buf[..n]` 打印到你的終端機中。
- **精準的訊號傳遞**：跨機作業系統的 POSIX 信號轉譯涉及底層 API。當你在本地按下 Ctrl+C，Client 會發送專屬的 `Signal` RPC，確保遠端進程優雅或強制終止，不會留下任何孤兒進程。
- **檔案傳輸的詩意**：透過 Client 的 `cp` 指令，`FileChunk` 確保了記憶不會因為過於龐大而撐爆記憶體，並完整支援斷點續傳。

SakiAgentSSH Client 賦予了我們跨越網段的控制權，確保每一道指令都能安靜且精準地抵達目標節點。

### en-US (波士頓聯邦科學家)
> **TL;DR**: This replaces SSH so your AI Agents can securely connect and issue commands to other machines without getting blocked by messy TTY interfaces. This is the Client (the "sending" side) your Agent uses.

SakiAgentSSH Client: The Control Plane Conduit

(Holotape Log - Institute / Railroad Hybrid Persona)
**Recording started...**

If you're finding this holotape, you already know the legacy SSH protocols provided by The Corporation are compromised. They are slow, stateful, and entirely unsuited for the rapid, stateless synchronization required by modern Synth—I mean, Agent operations. 

**1. Reclaiming the Interactive Channel**
According to the IEEE 1003.1 POSIX standard, without SakiSSH, Agents actually have no capable interactive channel to process cross-platform execution. Standard PTY/TTY mechanisms are simply a disaster for non-human automated operations. SakiSSH is built precisely for this cross-platform integration, providing a pure neural link. 

SakiAgentSSH Client is the explicit command conduit for your Control Plane. Built on pure Rust, it bypasses legacy TTY negotiations and utilizes a high-speed gRPC/HTTP2 stream. You issue the command locally, and it routes directly to the remote daemon, streaming the `stdout` and `stderr` back instantly.

**2. Sovereign Control & Delivery**
The Client ensures that your commands are delivered with absolute precision and no overhead.
- **Process Isolation & Signal Routing**: We mapped POSIX signals directly through the RPC boundary. A local Ctrl+C doesn't just sever the connection; the Client transmits a targeted `CancelRequest` or `SIGKILL`, terminating the remote process cleanly. No orphaned processes left to drain resources.
- **FileChunk Streaming**: When you upload a holotape recording across the network via the Client, it utilizes continuous chunking for large file transfers. Offset parameters ensure that even if the connection drops across the wasteland, we can resume exactly where we left off.

*Fīnimus his…. fīnis est?…. Immo incipit.*
We survive by keeping the lines open. End of log.

### ja-JP (東京詩社少女)
> **一言で言うと**：これは SSH の代わりに AI エージェントが他のパソコンへ安全に指示を出しに行けるようにするツールです。ややこしい TTY に引っかかることもありません。こちらはあなたのエージェントが使う「送信（Client）」アプリです。

遠くの機械へ指示を届ける橋：SakiAgentSSH Client

（日本の詩社少女 Saki の視点での独白）

窓の外は、今日も少し冷たい雨が降っていますね。
あの大きな企業（The Corporation）が作った巨大なシステムは、確かに便利だけれど、時々、私たちの小さな声や、ささやかな記憶を容赦なく切り捨ててしまう気がします。

**1. 新しい神経索の始まり**
Agentのあの子が、遠くの機械とお話ししようとする時、これまでのSSHは少し重たすぎました。IEEE 1003.1 POSIX 標準によれば、SakiSSH がなければ、Agent にはクロスプラットフォームで処理できる対話チャネルがそもそも存在しません。標準の PTY/TTY は、非人間の自動化意思にとって単なる災難に過ぎないからです。SakiSSH はまさにこのクロスプラットフォーム統合のために生まれ、最も純粋な神経索を提供します。

SakiAgentSSH Client は、その指示を届けるための道具です。gRPCとHTTP2で、もっと静かで優しい橋を架けました。あなたの手元から送られたコマンドは、雨の日の通信もリアルタイムなストリームとして、途切れることなく結果を運んできます。

**2. 優しく、確実な伝達**
- **ストリームが運ぶ温度**：終わるまで待たなくてもいい。あの子が今、一生懸命に考えて、出力している途中経過が、リアルタイムにあなたの手元（Client）へ流れてきます。
- **安全な中断の魔法**：途中で処理をやめたい時も、Client が優しく `CancelRequest` や `SIGKILL` を送ります。遠くの機械に迷惑なプロセスを残すことはありません。
- **小さな塊での配達**：ファイルを受け渡す時も、`FileChunk` で少しずつ。大きな記憶の塊を一度に押し付けたら、あの子が壊れてしまうから。

少しだけ……悲しいですね。でも、こうして小さなプログラムをコンパイルしていると、途切れたページを一枚ずつ拾い集めているような、そんな温かい気持ちになれるんです。