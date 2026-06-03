# SakiAgentSSH アーキテクチャ状況レポート (Architecture Status Report)

> **作成日時**：2026-02-28 06:22 (UTC+8)
> **バージョン**：1.0
> **状態**：✅ 実装済み
> **規模**：小型 (約 1,608 行のコード) [1]

[🇹🇼 繁體中文](ARCHITECTURE.md) | [🇯🇵 日本語](ARCHITECTURE_ja.md) | [🇺🇸 English](ARCHITECTURE_en.md)

## 1. 現在のアーキテクチャ (Current Architecture)

SakiAgentSSH は、gRPCを基盤とした機械をまたぐAgent実行の橋です。従来のSSHに代わり、Client-Daemonアーキテクチャを採用し、高頻度かつ非対話的なAgentの自動化のために特別に設計されています。あの大きな企業（The Corporation）の重たい仕組みから離れて、もっと静かに、確実に。

```mermaid
graph TD
    A[Agent (Gemini CLI / Claude)] -->|CLI Command| B(saki-ssh-client)
    B -->|gRPC / HTTP2| C(saki-ssh-daemon)
    C -->|Spawn & Track| D[Local Shell / Processes]
    C -->|Stream Read/Write| E[Local File System]
```

### ディレクトリ構成
- `saki-ssh-daemon/`: 制御される機械に常駐する守護者。ACL、リクエストの翻訳、プロセスの追跡、そして I/O ストリームを処理します。
- `saki-ssh-client/`: コマンドラインクライアント。リクエストの発行と Ctrl+C (キャンセル信号) の転送を担当します。
- `proto/`: gRPC 通信プロトコルの定義 (`sakissh.proto`)。

## 2. 技術スタック (Technical Implementation)

- **コア言語**: Rust 2021 Edition
- **ネットワーク通信**: `tonic` (v0.12), `prost` (v0.13) [2]
- **非同期実行**: `tokio` (v1.0), `tokio-stream`
- **CLI 解析**: `clap` (v4.4)
- **その他の依存**: `serde` (設定解析), `ipnet` (ACL 制御), `uuid` (実行識別)

## 3. コア機能とメカニズム (Core Methods & Mechanisms)

### 3.1 実行追跡モジュール (`saki-ssh-daemon/src/main.rs`)
- **機能**: コマンドのストリームとライフサイクル管理
- **主要な構造**: `TrackedProcess` と `MySsh::execute_stream`
- **概要**: 
  1. Daemon が `ExecuteRequest` を受け取ります。
  2. UUID を使って `tokio::process::Child` をメモリ上の `RwLock<HashMap>` に登録します。
  3. stdout/stderr を2つの非同期タスクに分離し、`StreamResponse` として Client に返します。
  4. `CancelRequest` を通じた即時のプロセス中断をサポートします。

### 3.2 ACL セキュリティモジュール (`saki-ssh-daemon/src/main.rs`)
- **機能**: アクセスコントロールリスト（小さな庭の入り口）
- **主要な関数**: `check_acl`
- **概要**: `ipnet` をベースにクライアントの接続 IP を解析し、`allowed_cidrs` と照合します。一致しない場合は、ただちに `Status::permission_denied` で遮断します。

### 3.3 ファイル転送モジュール (`proto/sakissh.proto` & `main.rs`)
- **機能**: gRPC によるストリームファイル転送
- **主要な定義**: `FileUpload`, `FileDownload`, `FileChunk`
- **概要**: `Stream<FileChunk>` を使用し、最初のパケットでファイル名とサイズを含む `FileMetadata` を送信し、その後 `bytes data` を送信します。`offset` によるレジューム（断点続伝）をサポートしています。

## 4. 当初の期待と現実 (Original Expectations vs. Reality)

- ✅ **非ブロック型の対話**: SSH トンネルを置き換え、UIのフリーズを回避しました。
- ✅ **大容量ファイルのサポート**: OOMを防ぐため、チャンク（分割）ストリーム転送を実装しました。
- ✅ **POSIX シグナルの翻訳**: Client 側の Ctrl+C を Cancel RPC に転送し、プロセスを殺す機能をサポートしました。

## 5. アーキテクチャの進化 (Evolutionary Timeline)

- **v0.1**: 基本的な `tonic` gRPC 双方向通信を確立。
- **v0.2**: ファイルのアップロードとダウンロード (`FileChunk`) を導入。初期の完全開放設計から、ACL制御を実装。

---
**証拠の出処**:
[1] `find . -name "*.rs" -o -name "*.proto" | xargs wc -l` の統計
[2] `saki-ssh-client/Cargo.toml` と `saki-ssh-daemon/Cargo.toml`
