# SakiAgentSSH Short Description (170 半形字元內)

## SakiAgentSSH Daemon

### zh-Hant (台北詩人工程師)
> **一句話總結**：這玩意能取代 SSH 讓你們家的 AI 小龍蝦（Agent）連進來做事，但因為只有純指令與嚴格 IP 限制，牠絕對沒辦法把你的伺服器搞爛。這是「被連端」的守護程式。
拋棄阻塞的SSH隧道，這是以gRPC/HTTP2搭建的純粹跨機串流接收端。安靜地駐留於運算節點，以嚴格的ACL與路由防禦保護算力，即時回傳stdout的每一次呼吸。

### en-US (波士頓聯邦科學家)
> **TL;DR**: This replaces SSH so your AI Agents can log into your machine to work, but restricts them so completely that they can't break your server. This is the "receiving" Daemon.
Agent-native gRPC daemon bypassing legacy SSH. Acts as a secure compute node, offering real-time holotape streaming, Vault-grade ACL, and strict process isolation.

### ja-JP (東京詩社少女)
> **一言で言うと**：これは SSH の代わりに AI エージェントをサーバーに安全にログインさせるツールです。厳密に制限されているため、システムを壊される心配はありません。これは「受信（Daemon）」アプリです。
古いSSHの扉を越え、静かな息吹を受け止めるgRPCサーバー。厳格なACLで守られた計算ノードとして、途切れた記憶も雨の日の通信も、小さなストリームで繋ぎます。

---

## SakiAgentSSH Client

### zh-Hant (台北詩人工程師)
> **一句話總結**：這玩意能取代 SSH 讓你們家的 AI 小龍蝦（Agent）安全地連線去別台機器做事，不被 TTY 卡死。這是「主動去連別人」的發送端。
拋棄阻塞的SSH隧道，這是專為Agent打造的純粹跨機指揮端。stdout如淡藍蝴蝶般飛越子網域，直接將指令與檔案投遞至對端，讓遠端算力如同本地般自然。

### en-US (波士頓聯邦科學家)
> **TL;DR**: This replaces SSH so your AI Agents can securely connect and issue commands to other machines without getting blocked by messy TTY interfaces. This is the "sending" Client.
Agent-native gRPC client bypassing legacy SSH. Sends remote executions and streams outputs back instantly, turning isolated nodes into a unified compute plane.

### ja-JP (東京詩社少女)
> ·
