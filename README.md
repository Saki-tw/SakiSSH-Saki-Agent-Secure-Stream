<div align="center">

<img src="release/icon.png" width="128" alt="SakiAgentSSH">

# SakiAgentSSH — SASS Protocol v1.4

**IETF Internet-Draft: draft-sakistudio-sass-00**

![IETF](https://img.shields.io/badge/IETF-draft--sakistudio--sass--00-blue?style=flat-square)
![SASS](https://img.shields.io/badge/SASS-v1.4-DA70D6?style=flat-square)
![Protocol](https://img.shields.io/badge/protocol-gRPC%2FHTTP2-00CED1?style=flat-square)
![Rust](https://img.shields.io/badge/Rust-1.95+-orange?style=flat-square&logo=rust)
![Go](https://img.shields.io/badge/Go-1.22+-00ADD8?style=flat-square&logo=go)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-brightgreen?style=flat-square)
![License](https://img.shields.io/badge/license-Saki%20Studio-DA70D6?style=flat-square)
[![Website](https://img.shields.io/badge/website-saki--studio.com.tw-00CED1?style=flat-square)](https://saki-studio.com.tw)

[🇹🇼 繁體中文](#-繁體中文) · [🇺🇸 English](#-english) · [🇯🇵 日本語](#-日本語)

</div>

---

# 🇹🇼 繁體中文

## SakiAgentSSH — SASS Protocol v1.4

**IETF Internet-Draft: draft-sakistudio-sass-00**

### 簡介

SASS（Saki Agent Secure Stream）是一個應用層疊加協議，專為自主 AI 代理之間的已認證遠端指令執行、串流程序 I/O 及二進位檔案傳輸而設計。有鑑於自主 AI 編程代理在遠端機器上執行任務所引入之「流氓代理」(Rogue Agent) 威脅模型——不同於人類操作之傳統 SSH 用戶端，代理可能自主執行破壞性命令、竊取憑證或橫向滲透網路——SASS 協議透過「控制—傳輸解耦」架構，定義了一套傳輸無關的抽象訊息模型 (SAMM)，以 CBOR (RFC 8949) 與 JSON 作為基線序列化格式，實現嚴格的自包含性與 IETF 標準相容性。

SASS v1.4 達成了 AES（Almost Everywhere Superior）里程碑：基於二階隨機優越性 (SSD) 的版本間比較性宣稱，每次版本迭代均從代理概率空間中消除特定行為分支，同時維持期望損失，產生嚴格的 SSD 改善。

### 核心特性

| 特性 | 說明 |
|------|------|
| **6-Response 狀態機** | 所有可能的 Agent 行為收斂至 R1~R6 六種確定性回應，每一種皆保證儲存安全 |
| **多層主動威脅防禦 (MAS)** | 13Policy 啟發式邊界裁定器 + 認知挑戰機制 + 雙標準執行 |
| **Safety Gradient** | 七層損失邊界理論——每一協議層在上方所有層被攻破時，仍約束最壞情況損失 |
| **Transparent Branching** | 對 Agent 透明的寫入隔離，所有寫入操作重導至可丟棄分支目錄 |
| **Vi Swap** | 針對已認證但越權之 Agent，將其陷入模擬互動終端狀態，使 LLM 停止生成 |
| **Tarpit** | 針對未認證攻擊者，以 O(1) 記憶體成本串流高熵資料耗盡其資源 |
| **PTY Ring Buffer** | 環形緩衝區 + 偏移續傳，實現傳輸中斷後之冪等重連 |
| **控制—傳輸解耦** | SAMM 抽象訊息模型，支援 gRPC-h2 / WebSocket / TCP-CBOR-RPC 傳輸設定檔 |

### 架構圖

```
+----------------------------------------------------------+
| Layer 7: Transparent Branching (UVSF | Micro Branch)     |
+----------------------------------------------------------+
| Layer 6: Storage Sandbox (UVSF Core | KFS Kernel)        |
+----------------------------------------------------------+
| Layer 5: Forward-Secure Audit Trail (Hash Chain)          |
+----------------------------------------------------------+
| Layer 4: Capability & Session Management                  |
+----------------------------------------------------------+
| Layer 3: Active Threat Defense (13Policy, Tarpit, Vi)     |
+----------------------------------------------------------+
| Layer 2: Payload Encoding (Zstd Stream + Base64)          |
+----------------------------------------------------------+
| Layer 1: Abstract Transport Adapter (SAMM Interface)      |
+----------------------------------------------------------+
|  [Transport Profiles: gRPC-h2 | WS | TCP-CBOR-RPC]       |
+----------------------------------------------------------+

Orthogonal: 6-Response State Machine
+----------------------------------------------------------+
| R1: EXECUTE  | R2: CHALLENGE | R3: THROTTLE              |
| R4: VI_SWAP  | R5: TARPIT    | R6: DROP                  |
+----------------------------------------------------------+
```

### 建置指南

```bash
# 前置需求
# - Rust 1.95+ (rustup)
# - protoc 35.0+ (Protocol Buffers compiler)

# Daemon (伺服端)
cd saki-ssh-daemon && cargo build --release

# Client (用戶端)
cd saki-ssh-client && cargo build --release

# macOS GUI 應用程式 (SwiftUI)
cd SakiAgentSSH-Daemon && xcodegen generate && xcodebuild build -configuration Release
cd SakiAgentSSH-Client && xcodegen generate && xcodebuild build -configuration Release
```

詳細建置說明請參閱 [BUILDING_zh-TW.md](BUILDING_zh-TW.md)。

### 文件連結

| 資源 | 連結 |
|------|------|
| IETF Datatracker | https://datatracker.ietf.org/doc/draft-sakistudio-sass/ |
| 官方網站 | https://saki-studio.com.tw/sakiagentssh/ |
| RFC 全文 (本地) | [docs/draft-sakistudio-sass-00.txt](docs/draft-sakistudio-sass-00.txt) |
| 架構說明 | [ARCHITECTURE.md](ARCHITECTURE.md) |

### 作者

- **Hua Chang** — Saki Studio · Taipei, Taiwan
- **Claude Opus 4.6** — Anthropic · AI Co-author

### 版權聲明

Copyright © 2026 Saki Studio. All Rights Reserved.
協議規格文件另受 IETF Trust 授權約束。
詳見 [LICENSE](LICENSE)。

---

# 🇺🇸 English

## SakiAgentSSH — SASS Protocol v1.4

**IETF Internet-Draft: draft-sakistudio-sass-00**

### Overview

SASS (Saki Agent Secure Stream) is an application-layer overlay protocol for authenticated remote command execution, streaming process I/O, and binary file transfer between trusted agents. The proliferation of autonomous AI-powered coding agents operating on remote machines introduces a critical threat model: the Rogue Agent. Unlike traditional SSH clients controlled by human operators, agents may autonomously execute destructive commands, exfiltrate credentials, or pivot laterally across networks without explicit human authorization.

To address this, SASS defines a decoupled "Control-Transport Decoupling" architecture with an abstract SASS Abstract Messaging Model (SAMM) utilizing standard CBOR (RFC 8949) and JSON as baseline serializations, achieving strict self-containment and IETF standard compatibility.

SASS v1.4 achieves the AES (Almost Everywhere Superior) milestone: a comparative claim between protocol versions based on Second-order Stochastic Dominance (SSD). Each version iteration eliminates specific behavioral branches from the agent probability space while maintaining expected loss, yielding strict SSD improvement.

### Core Features

| Feature | Description |
|---------|-------------|
| **6-Response State Machine** | All possible Agent behaviors converge to R1~R6 deterministic responses, each guaranteeing storage safety |
| **Multi-layer Active Defense (MAS)** | 13Policy heuristic boundary adjudicator + cognitive challenges + dual standard enforcement |
| **Safety Gradient** | 7-layer loss bounding theory — each layer bounds worst-case loss even if all layers above are compromised |
| **Transparent Branching** | Write isolation invisible to the Agent; all writes redirected to a discardable branch directory |
| **Vi Swap** | Traps authenticated but unauthorized Agents in simulated interactive terminal state, halting LLM generation |
| **Tarpit** | Streams high-entropy data to unauthenticated attackers with O(1) daemon memory cost |
| **PTY Ring Buffer** | Ring buffer + offset resumption for idempotent reconnection after transport disruption |
| **Control-Transport Decoupling** | SAMM abstract messaging model supporting gRPC-h2 / WebSocket / TCP-CBOR-RPC transport profiles |

### Architecture

```
+----------------------------------------------------------+
| Layer 7: Transparent Branching (UVSF | Micro Branch)     |
+----------------------------------------------------------+
| Layer 6: Storage Sandbox (UVSF Core | KFS Kernel)        |
+----------------------------------------------------------+
| Layer 5: Forward-Secure Audit Trail (Hash Chain)          |
+----------------------------------------------------------+
| Layer 4: Capability & Session Management                  |
+----------------------------------------------------------+
| Layer 3: Active Threat Defense (13Policy, Tarpit, Vi)     |
+----------------------------------------------------------+
| Layer 2: Payload Encoding (Zstd Stream + Base64)          |
+----------------------------------------------------------+
| Layer 1: Abstract Transport Adapter (SAMM Interface)      |
+----------------------------------------------------------+
|  [Transport Profiles: gRPC-h2 | WS | TCP-CBOR-RPC]       |
+----------------------------------------------------------+

Orthogonal: 6-Response State Machine
+----------------------------------------------------------+
| R1: EXECUTE  | R2: CHALLENGE | R3: THROTTLE              |
| R4: VI_SWAP  | R5: TARPIT    | R6: DROP                  |
+----------------------------------------------------------+
```

### Build Instructions

```bash
# Prerequisites
# - Rust 1.95+ (rustup)
# - protoc 35.0+ (Protocol Buffers compiler)

# Daemon (server-side)
cd saki-ssh-daemon && cargo build --release

# Client (client-side)
cd saki-ssh-client && cargo build --release

# macOS GUI apps (SwiftUI)
cd SakiAgentSSH-Daemon && xcodegen generate && xcodebuild build -configuration Release
cd SakiAgentSSH-Client && xcodegen generate && xcodebuild build -configuration Release
```

For detailed build instructions, see [BUILDING.md](BUILDING.md).

### Documentation

| Resource | Link |
|----------|------|
| IETF Datatracker | https://datatracker.ietf.org/doc/draft-sakistudio-sass/ |
| Official Website | https://saki-studio.com.tw/sakiagentssh/ |
| Full RFC (local) | [docs/draft-sakistudio-sass-00.txt](docs/draft-sakistudio-sass-00.txt) |
| Architecture | [ARCHITECTURE_en.md](ARCHITECTURE_en.md) |

### Authors

- **Hua Chang** — Saki Studio · Taipei, Taiwan
- **Claude Opus 4.6** — Anthropic · AI Co-author

### Copyright

Copyright © 2026 Saki Studio. All Rights Reserved.
Protocol specification documents are additionally subject to the IETF Trust license.
See [LICENSE](LICENSE) for details.

---

# 🇯🇵 日本語

## SakiAgentSSH — SASS Protocol v1.4

**IETF Internet-Draft: draft-sakistudio-sass-00**

### 概要

SASS（Saki Agent Secure Stream）は、信頼されたエージェント間の認証付きリモートコマンド実行、ストリーミングプロセス I/O、およびバイナリファイル転送のためのアプリケーション層オーバーレイプロトコルです。自律的な AI コーディングエージェントがリモートマシン上で動作することにより、従来の人間が操作する SSH クライアントとは異なる「ローグエージェント」(Rogue Agent) という脅威モデルが生まれました。エージェントは人間の明示的な許可なく、破壊的なコマンドの実行、認証情報の窃取、またはネットワーク内の横方向移動を自律的に行う可能性があります。

これに対処するため、SASS は「制御・トランスポート分離」アーキテクチャを定義し、標準的な CBOR (RFC 8949) と JSON をベースラインのシリアライゼーションとして利用する SASS 抽象メッセージングモデル (SAMM) により、厳密な自己完結性と IETF 標準との互換性を実現します。

SASS v1.4 は AES（Almost Everywhere Superior）マイルストーンを達成しました：二次確率的優越性 (SSD) に基づくプロトコルバージョン間の比較的主張であり、各バージョンの反復はエージェント確率空間から特定の行動分岐を排除しながら期待損失を維持し、厳密な SSD 改善をもたらします。

### コア機能

| 機能 | 説明 |
|------|------|
| **6-Response ステートマシン** | すべての Agent の行動が R1〜R6 の6つの決定論的応答に収束し、各応答がストレージの安全性を保証 |
| **多層アクティブ防御 (MAS)** | 13Policy ヒューリスティック境界裁定器 + 認知チャレンジメカニズム + デュアルスタンダード施行 |
| **Safety Gradient** | 7層損失バウンディング理論——上位すべての層が侵害されても、各層が最悪ケースの損失を制約 |
| **Transparent Branching** | Agent に透明な書き込み分離。すべての書き込み操作が破棄可能なブランチディレクトリにリダイレクト |
| **Vi Swap** | 認証済みだが権限外の Agent をシミュレートされた対話ターミナル状態に閉じ込め、LLM の生成を停止 |
| **Tarpit** | 未認証の攻撃者に O(1) デーモンメモリコストで高エントロピーデータをストリーミングし、リソースを消耗 |
| **PTY リングバッファ** | リングバッファ + オフセット再開による、トランスポート中断後の冪等な再接続 |
| **制御・トランスポート分離** | SAMM 抽象メッセージングモデル。gRPC-h2 / WebSocket / TCP-CBOR-RPC トランスポートプロファイル対応 |

### アーキテクチャ

```
+----------------------------------------------------------+
| Layer 7: Transparent Branching (UVSF | Micro Branch)     |
+----------------------------------------------------------+
| Layer 6: Storage Sandbox (UVSF Core | KFS Kernel)        |
+----------------------------------------------------------+
| Layer 5: Forward-Secure Audit Trail (Hash Chain)          |
+----------------------------------------------------------+
| Layer 4: Capability & Session Management                  |
+----------------------------------------------------------+
| Layer 3: Active Threat Defense (13Policy, Tarpit, Vi)     |
+----------------------------------------------------------+
| Layer 2: Payload Encoding (Zstd Stream + Base64)          |
+----------------------------------------------------------+
| Layer 1: Abstract Transport Adapter (SAMM Interface)      |
+----------------------------------------------------------+
|  [Transport Profiles: gRPC-h2 | WS | TCP-CBOR-RPC]       |
+----------------------------------------------------------+

直交: 6-Response ステートマシン
+----------------------------------------------------------+
| R1: EXECUTE  | R2: CHALLENGE | R3: THROTTLE              |
| R4: VI_SWAP  | R5: TARPIT    | R6: DROP                  |
+----------------------------------------------------------+
```

### ビルド手順

```bash
# 前提条件
# - Rust 1.95+ (rustup)
# - protoc 35.0+ (Protocol Buffers コンパイラ)

# Daemon（サーバー側）
cd saki-ssh-daemon && cargo build --release

# Client（クライアント側）
cd saki-ssh-client && cargo build --release

# macOS GUI アプリケーション (SwiftUI)
cd SakiAgentSSH-Daemon && xcodegen generate && xcodebuild build -configuration Release
cd SakiAgentSSH-Client && xcodegen generate && xcodebuild build -configuration Release
```

詳細なビルド手順については [BUILDING_ja.md](BUILDING_ja.md) をご参照ください。

### ドキュメント

| リソース | リンク |
|----------|--------|
| IETF Datatracker | https://datatracker.ietf.org/doc/draft-sakistudio-sass/ |
| 公式ウェブサイト | https://saki-studio.com.tw/sakiagentssh/ |
| RFC 全文（ローカル） | [docs/draft-sakistudio-sass-00.txt](docs/draft-sakistudio-sass-00.txt) |
| アーキテクチャ | [ARCHITECTURE_ja.md](ARCHITECTURE_ja.md) |

### 著者

- **Hua Chang** — Saki Studio · 台北、台湾
- **Claude Opus 4.6** — Anthropic · AI 共著者

### 著作権表示

Copyright © 2026 Saki Studio. All Rights Reserved.
プロトコル仕様書は IETF Trust ライセンスにも従います。
詳細は [LICENSE](LICENSE) をご覧ください。

---

## Related Projects / 関連プロジェクト

| Project | Description | Links |
|---------|-------------|-------|
| [SakiMCP](https://github.com/saki-tw/SakiMCP) | Model Context Protocol server (Rust) | [App Store](https://apps.apple.com/tw/app/sakimcp/id6758668850?mt=12) |
| [SakiAgentSkills](https://github.com/saki-tw/SakiAgentSkills) | Agent skill library (Swift/Python) | [App Store](https://apps.apple.com/tw/app/saki-agent-skills/id6758680481?mt=12) |
| [VI-SakiWin64](https://github.com/Saki-tw/VI-SakiWin64) | Vi editor for Windows x64 | [Winget](https://github.com/microsoft/winget-pkgs/tree/master/manifests/s/SakiStudio/SakiVi) |

---

<div align="center">

*在這個終端機總是漆黑一片的冷酷年代裡，這抹顏色就像是一隻淡藍色的蝴蝶自肚腹裡升起。*
*In this cold age where terminals are always pitch black, this color rises like a pale blue butterfly from within.*
*端末がいつも真っ暗なこの冷たい時代に、この色は腹の底から舞い上がる淡い蝶のように。*

**Saki Studio** · [saki-studio.com.tw](https://saki-studio.com.tw) · [GitHub](https://github.com/saki-tw)

</div>
