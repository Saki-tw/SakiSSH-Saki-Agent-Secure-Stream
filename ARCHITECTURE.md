# SakiAgentSSH 架構現況報告 (Architecture Status Report)

> **建立時間**：2026-02-28 06:22 (UTC+8)
> **版本**：1.0
> **狀態**：✅ 已實作
> **規模**：微型 (約 1,608 行原始碼) [1]

[🇹🇼 繁體中文](ARCHITECTURE.md) | [🇯🇵 日本語](ARCHITECTURE_ja.md) | [🇺🇸 English](ARCHITECTURE_en.md)

## 1. 專案目前架構 (Current Architecture)

SakiAgentSSH 是一個基於 gRPC 的跨機 Agent 執行橋樑。採用 Client-Daemon 架構，取代傳統 SSH，專為高頻次、非互動式的 Agent 自動化所設計。

```mermaid
graph TD
    A[Agent (Gemini CLI / Claude)] -->|CLI Command| B(saki-ssh-client)
    B -->|gRPC / HTTP2| C(saki-ssh-daemon)
    C -->|Spawn & Track| D[Local Shell / Processes]
    C -->|Stream Read/Write| E[Local File System]
```

### 目錄映射
- `saki-ssh-daemon/`: 守護進程，駐留於受控機。處理 ACL, 請求轉譯, 流程追蹤與 I/O 串流。
- `saki-ssh-client/`: 命令列客戶端，負責發起請求與轉發 Ctrl+C (取消信號)。
- `proto/`: gRPC 通訊協定定義 (`sakissh.proto`)。

## 2. 技術實作堆疊 (Technical Implementation)

- **核心語言**: Rust 2021 Edition
- **網路通訊**: `tonic` (v0.12), `prost` (v0.13) [2]
- **非同步執行**: `tokio` (v1.0), `tokio-stream`
- **CLI 解析**: `clap` (v4.4)
- **其他依賴**: `serde` (配置解析), `ipnet` (ACL 控制), `uuid` (執行識別)

## 3. 核心方法與機制 (Core Methods & Mechanisms)

### 3.1 執行追蹤模組 (`saki-ssh-daemon/src/main.rs`)
- **功能模組**: 指令串流與生命週期管理
- **關鍵結構**: `TrackedProcess` 與 `MySsh::execute_stream`
- **邏輯摘要**: 
  1. Daemon 收到 `ExecuteRequest`。
  2. 使用 UUID 註冊 `tokio::process::Child` 至記憶體 `RwLock<HashMap>`。
  3. 將 stdout/stderr 分離為兩個非同步任務，以 `StreamResponse` 傳回 Client。
  4. 支援透過 `CancelRequest` 立即中斷進程。

### 3.2 ACL 安全模組 (`saki-ssh-daemon/src/main.rs`)
- **功能模組**: 存取控制清單
- **關鍵函式**: `check_acl`
- **邏輯摘要**: 基於 `ipnet` 解析客戶端連線 IP，比對 `allowed_cidrs`。若不符則直接阻斷 `Status::permission_denied`。

### 3.3 檔案傳輸模組 (`proto/sakissh.proto` & `main.rs`)
- **功能模組**: gRPC 串流檔案傳輸
- **關鍵定義**: `FileUpload`, `FileDownload`, `FileChunk`
- **邏輯摘要**: 使用 `Stream<FileChunk>`，首封包傳送 `FileMetadata` (包含檔名與大小)，後續傳送 `bytes data`。支援 `offset` 進行斷點續傳。

## 4. 原始預期與偏差 (Original Expectations vs. Reality)

- ✅ **非阻塞互動**: 取代了 SSH 隧道，避免介面卡頓。
- ✅ **大檔傳輸支援**: 已實作分塊串流傳輸 (chunking) 避免 OOM。
- ✅ **POSIX 信號轉譯**: 支援 Client 端的 Ctrl+C 轉發為 Cancel RPC 殺死進程。

## 5. 架構演進軌跡 (Evolutionary Timeline)

- **v0.1**: 建立基本的 `tonic` gRPC 雙向傳輸。
- **v0.2**: 引入檔案上傳與下載 (`FileChunk`)。實作 ACL 控制，取代早期全開放設計。

---
**證據來源**:
[1] `find . -name "*.rs" -o -name "*.proto" | xargs wc -l` 統計
[2] `saki-ssh-client/Cargo.toml` 與 `saki-ssh-daemon/Cargo.toml`
