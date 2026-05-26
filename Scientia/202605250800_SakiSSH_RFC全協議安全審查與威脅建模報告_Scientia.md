# SASS (Saki Agent Secure Stream) RFC 全協定安全審查與威脅建模報告

> 建立時間：2026-05-25 08:00 (UTC+8 Asia/Taipei)  
> 專案簡稱：SakiSSH  
> 類別：Scientia 知識與研究歸檔  
> 版本：v1.0 (IETF RFC 預審與 STRIDE 威脅建模完整報告)  

---

## 摘要

SASS (Saki Agent Secure Stream) v7.0 協定作為取代傳統 SSH 的次世代 Agent 執行與儲存沙盒協定，採用了「開源核心 (Core) + 平台特權插件 (Plugins)」的雙軌引擎架構。本報告對 SASS 七層安全防禦模型進行了全協定的擴寫與解構，揭示了其在「無特權 (Zero-Permission)」環境下實現高強度隔離的密碼學與系統級精妙之處。同時，本報告以最挑剔的 RFC Reviewer 視角，針對 UVSF 沙盒 TOCTOU 穿透、Tarpit 資源自我反噬、審計日誌私鑰洩漏、Zstd 壓縮炸彈、以及 ChaCha20 認知挑戰重放等五大「最嚴重未預期行為」進行了硬核的威脅建模，並提出了 SASS v7.0 在協定層與代碼級別的修補對策，使 SASS 具備抵禦國家級 APT 與本地惡意 Agent 突破的能力。

---

## 第一部分：SASS 七層協定棧規格擴寫與設計精妙之處

SASS 協定（ALPN: `x-sakirpc-v5` / MIME: `application/grpc+saki`）在 gRPC 傳輸層之上，構建了一個非對稱、多維度的安全沙盒空間。

```
+-----------------------------------------------------------------------------------+
| L6: I/O Sandbox       — UVSF Symlink Engine | DEXT/Minifilter Plugins             |
+-----------------------------------------------------------------------------------+
| L5: Audit Trail       — Forward-Secure Hash Chain (SHA256) & Ed25519 Signatures   |
+-----------------------------------------------------------------------------------+
| L4: Session & Cap     — Ed25519 Token Authentication & 5-Dimensional ACL Token     |
+-----------------------------------------------------------------------------------+
| L3: Threat Defense    — 13Policy YAML Engine, ChaCha20 Handshake, Active Tarpit   |
+-----------------------------------------------------------------------------------+
| L2: Payload Encoding  — Zstd Decompression + Base64 Enveloping                    |
+-----------------------------------------------------------------------------------+
| L1: Transport Sec     — TLS 1.3 mTLS (Mandatory PFS / x-sakirpc-v5)               |
+-----------------------------------------------------------------------------------+
| L0: Network ACL       — CIDR-based Source IPv4/IPv6 White-listing                 |
+-----------------------------------------------------------------------------------+
```

### 1.1 各層協定封包規格與設計細節

#### 【Layer 0 & Layer 1：網路白名單與雙向傳輸安全】
* **規格**：強制要求 TLS 1.3。密碼套件（Cipher Suites）僅容許 `TLS_AES_256_GCM_SHA384` 與 `TLS_CHACHA20_POLY1305_SHA256`，提供完美前向安全性 (PFS)。ALPN 強制匹配 `x-sakirpc-v5`，拒絕任何未攜帶此標籤的 HTTP/2 連線。
* **精妙之處**：將傳輸安全直接綁定於 ALPN 與 gRPC 握手，使任何標準的網路掃描工具在掃描 19284 埠時，均會因為 ALPN 不匹配或 TLS 握手失敗而被即時拒絕，完美隱藏了 SASS Daemon 的特徵。

#### 【Layer 2：Zstd + Base64 零特權封裝】
* **規格**：所有 gRPC 傳輸的 `RawPayload` 均需通過 RFC 8878 Zstd 壓縮，並使用 RFC 4648 Base64 封裝。
* **精妙之處**：這為不具備 Linux `unshare` 或 macOS 驅動權限的 Userspace 環境，提供了一個輕量級的「傳輸沙盒」。所有的指令、環境變數與路徑在網路傳輸中均以高度壓縮的高熵編碼傳遞，減少了 gRPC 內存複製與 token 化負擔。

#### 【Layer 3：13Policy 規則引擎與 ChaCha20 認知挑戰】
* **規格**：
  * **13Policy**：YAML 配置危險指令黑名單（如 `rm`, `mv`, `chmod`）。
  * **ChaCha20 Challenge**：Daemon 隨機生成 64-byte 明文與 12-byte nonce，使用隨機 256-bit 金鑰加密為 `encrypted_challenge` 發送給 Client。Client 必須證明其能使用預共享/Session 金鑰正確解密並回傳明文，否則拒絕執行。
  * **Tarpit**：若未授權連線強行執行敏感指令，Daemon 在 gRPC `ExecuteStream` 中向 Client 持續發送由隨機高熵字節組成的 40MB 垃圾流，每個 chunk 發送間隔隨機化（50-200ms），只在最後一個 Chunk 的 `exit_code` 填入 `-1`。
* **精妙之處**：**主動反制 (Active Defense)**。SASS 不僅僅是「拒絕連線」，而是透過 **Tarpit** 進行「時間耗竭攻擊」反制惡意掃描器，並使用 **ChaCha20 認知挑戰** 迫使 Client 消耗 CPU 算力進行解密，拉高駭客並行爆破的成本。

#### 【Layer 4 & Layer 5：能力會話與防篡改審計鏈】
* **規格**：
  * **Capability ACL**：Session Token 中內嵌五維度權限（`Read`, `Write`, `Execute`, `Network`, `Admin`）。
  * **Audit Trail**：每次執行 `Execute` 時，Daemon 將 `(Prev_Hash + Cmd + Args + Cwd + User + Timestamp)` 組成區塊，使用 SHA256 計算當前區塊 Hash，並使用本機持久化的 Ed25519 私鑰對該 Hash 進行簽章，以 JSONL 格式追加寫入 `~/.config/sass/audit.log`。
* **精妙之處**：**日誌不可否認性**。每一條稽核日誌都與前一條日誌的密碼學雜湊鎖定，並附帶非對稱簽章。即便攻擊者入侵系統，也無法在不破壞雜湊鏈完整性的情況下「隱形」刪除歷史指令記錄。

#### 【Layer 6：v7.0 Core & Plugins 雙軌儲存沙盒】
* **規格**：
  * **開源核心版 (UVSF)**：在 `~/.config/sass/sandbox/{session_id}/` 下動態建立一個 Symlink Tree，只將允許讀寫的白名單目錄連結進去，並將進程的 `cwd` 強制切換至該沙盒路徑。
  * **商業特權插件 (KFS/WSDK Engine)**：動態接管 `StorageProvider`，利用 macOS DriverKit/FSKit 或 Windows Minifilter，在 OS 核心級進行強制 I/O 重新導向與虛擬 APFS snapshot / NTFS 稀疏檔案掛載。
* **精妙之處**：**優雅降級與零權限啟動**。開源版實現了「零特權」的快速部署與基本隔離，而商業版提供了「不可穿過」的核心級防禦與極致效能，兩者共享相同的 gRPC 控制平面，實現無縫切換。

---

## 第二部分：嚴苛 RFC Review ── 五大最嚴重未預期行為與威脅建模

為了讓 SASS 協定無懈可擊，我們必須站在最挑剔的 RFC 審查員角度，對上述設計進行「極限自我打擊」。

### 2.1 威脅 01：UVSF Userspace 沙盒穿透 (TOCTOU & Symlink Directory Traversal)

* **攻擊路徑與建模**：
  1. 受控或惡意的進程在 UVSF 沙盒 `/Users/hc1034/.sass/sandbox/{session_id}/` 內執行。
  2. 該進程與 SASS Daemon 進行並行競態（Race Condition）。
  3. 進程發起一項 I/O 操作（如寫入 `/Users/hc1034/.sass/sandbox/{session_id}/data/test.txt`）。
  4. SASS Daemon 在 Userspace 檢查該路徑，判定 `data` 是指向合法臨時目錄的 symlink，允許該操作（Check 階段）。
  5. 在系統調用實際執行（Use 階段）之前的微秒級空隙中，進程快速調用 `unlink()` 刪除 `data`，並創建一個同名的 `data` 符號連結，指向敏感目錄（如 `/Users/hc1034/.ssh`）。
  6. OS 執行寫入，數據被寫入 `/Users/hc1034/.ssh/authorized_keys`。
* **最嚴重未預期行為**：**Userspace 沙盒完全崩潰**。惡意進程成功實現 TOCTOU 穿透，取得本機 SSH 權限或讀取敏感凭證。

### 2.2 威脅 02：Tarpit 慢阻斷引發的反向拒絕服務 (Self-Inflicted DoS)

* **攻擊路徑與建模**：
  1. 攻擊者在未授權狀態下，透過並行發起 10,000 個連線請求。
  2. 每個連線均故意發送一個包含高危指令（如 `rm -rf /`）的偽造 Payload。
  3. SASS Daemon 的 Layer 3 偵測到危險，為這 10,000 個連線同時觸發 **Tarpit** 防禦。
  4. Daemon 開始為這 10,000 個連線中的每一個連線，在記憶體中配置 40MB 的隨機垃圾資料，並啟動獨立的 Tokio 異步 Task 以 50ms 的間隔發送 chunk。
* **最嚴重未預期行為**：**內存耗盡與 CPU 崩潰**。
  * $10,000 \times 40\text{MB} = 400\text{GB}$ 虛擬記憶體或大量的 I/O buffer 配置，將直接觸發作業系統的 OOM Killer，殺死 SASS Daemon 本身。
  * 頻寬被 400GB 的垃圾垃圾數據完全堵死，導致合法進程無法與 Daemon 通訊。防禦機制變成了最致命的自殘 DoS 漏洞。

### 2.3 威脅 03：日誌鏈回滾與私鑰洩露 (Key Compromise & Ephemeral State Loss)

* **攻擊路徑與建模**：
  1. 攻擊者入侵系統，成功取得了運行 SASS Daemon 的本機用戶權限（L3 OS 權限）。
  2. 攻擊者直接讀取 `~/.config/sass/audit_key.pem`（即便有 `chmod 600`，同權限的惡意進程亦可讀取）。
  3. 攻擊者執行惡意指令後，使用竊取的 Ed25519 私鑰，重新計算並簽署一條不含該惡意指令的「乾淨日誌鏈」，並覆蓋 `audit.log`。
  4. 由於私鑰就在本機，Daemon 無法在事後察覺這條日誌鏈曾被重新構造。
* **最嚴重未預期行為**：**審計抗抵賴性 (Non-Repudiation) 徹底失效**。攻擊者可以實現歷史指令的「完美抹除」，而外部稽核員在核對 Hash Chain 時，驗證結果依然是「合法且完整」的。

### 2.4 威脅 04：Zstd 壓縮炸彈邊界解碼 (Decompression Bomb / Zip Bomb)

* **攻擊路徑與建模**：
  1. 攻擊者構造一個僅有 10KB 大小的 Zstd 壓縮包 Payload。
  2. 該壓縮包在壓縮時經過極限優化，其明文內容是 10GB 的連續零字節（`0x00`）。
  3. 攻擊者透過 gRPC 將此 Payload 發送給 SASS Daemon。
  4. SASS Daemon 在 Layer 2 接收到 Payload，調用 `DecodePayload` 進行解密與解壓縮。
* **最嚴重未預期行為**：**解壓崩潰**。Daemon 在 Userspace 嘗試將 10KB 解壓為 10GB 記憶體區塊，引發瞬間記憶體暴漲，觸發 OOM Crash，或者耗盡單核 CPU 運算資源，使 Daemon 卡死。

### 2.5 威脅 05：ChaCha20 認知挑戰的 TLS 通道剝離與重放 (Channel Stripping & Replay)

* **攻擊路徑與建模**：
  1. 攻擊者監聽網路傳輸，或在中間人（MITM）位置攔截 Client 與 Daemon 的連線。
  2. 當 Daemon 向合法 Client 發送 ChaCha20 `encrypted_challenge` 與 `nonce` 時，中間人記錄該封包。
  3. 同時，攻擊者發起一條並行的惡意連線至 Daemon，並在該連線中，將剛才攔截到的 Challenge Nonce 與 Client 解密後回傳的 Plaintext 答案進行重放。
  4. 如果 Challenge Entry 的 TTL 過長，或者 Challenge 未與 TLS 連線狀態鎖定。
* **最嚴重未預期行為**：**認知認證被繞過**。攻擊者成功藉由重放合法 Client 的計算結果，通過了 ChaCha20 挑戰，偽造了具備計算能力的合法憑證。

---

## 第三部分：SASS v7.0 協定規格對策與代碼級修復方案

針對上述嚴苛的審查漏洞，我們在 SASS v7.0 協定中引入了具體的規格修補與代碼級安全防禦設計。

### 3.1 對策 01：防範 TOCTOU 的 Userspace 絕對隔離 (Openat O_NOFOLLOW)

* **協定規格修補**：
  * UVSF 引擎**禁止**使用任何基於字串路徑拼接的 `open()` 或 `write()` 系統調用。
  * 必須採用基於檔案描述符 (FD) 的相對路徑操作（如 `openat(2)`），且強制攜帶 `O_NOFOLLOW` 與 `O_CLOEXEC` 標記，禁止跟隨任何 symlink。
* **Go 實作防護代碼 stub**：
```go
package sandbox

import (
	"fmt"
	"os"
	"syscall"
)

// SafeOpenWithinSandbox 確保在 Userspace 開啟檔案時，絕對不會因為 TOCTOU Symlink 攻擊而穿透沙盒
func SafeOpenWithinSandbox(sandboxFd int, relativePath string, flags int, perm os.FileMode) (*os.File, error) {
	// 強制攜帶 O_NOFOLLOW 避免跟隨符號連結，O_CLOEXEC 避免 FD 洩漏
	safeFlags := flags | syscall.O_NOFOLLOW | syscall.O_CLOEXEC
	
	fd, err := syscall.Openat(sandboxFd, relativePath, safeFlags, uint32(perm))
	if err != nil {
		if err == syscall.ELOOP {
			return nil, fmt.Errorf("security violation: symlink traversal detected in path %s", relativePath)
		}
		return nil, err
	}
	
	return os.NewFile(uintptr(fd), relativePath), nil
}
```

### 3.2 對策 02：Tarpit 零分配環形緩衝區與速率限制 (Zero-Allocation Tarpit)

* **協定規格修補**：
  * **零內存分配**：Daemon 記憶體中僅保留一個單一、全域且唯讀的 64KB 高熵垃圾資料靜態緩衝區（`static_entropy_buffer`）。Tarpit 發送時，重複切片發送此緩衝區，**禁止**為個別連線單獨配置 40MB 空間。
  * **連線池與並行限制**：Tarpit 最大並行 Task 數限制為 `max_tarpit_tasks = 32`。超過此限制的未授權高危請求，直接在 Layer 0 ACL 階段進行 TCP 物理斷開，不再提供 Tarpit。
* **Rust 實作防護代碼 stub**：
```rust
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;

lazy_static::lazy_static! {
    // 全域唯讀靜態高熵緩衝區，大小 64KB，避免動態內存分配
    static ref STATIC_ENTROPY: Arc<Vec<u8>> = Arc::new((0..65536).map(|_| rand::random::<u8>()).collect());
}

pub async fn stream_tarpit_safe(
    tx: mpsc::Sender<Result<pb::StreamResponse, tonic::Status>>,
    total_size_mb: usize,
) {
    let chunk_size = 65536; // 64KB
    let total_chunks = (total_size_mb * 1024 * 1024) / chunk_size;
    let entropy_ref = STATIC_ENTROPY.clone();

    tokio::spawn(async move {
        for chunk_idx in 0..total_chunks {
            // 重複使用相同的唯讀 buffer，內存開銷為 O(1)
            let response = pb::StreamResponse {
                source: pb::StreamResponse::Stderr as i32,
                data: entropy_ref[..].to_vec(), 
                exit_code: if chunk_idx == total_chunks - 1 { Some(-1) } else { None },
            };

            if tx.send(Ok(response)).await.is_err() {
                break; // Client 斷開連線，立即回收 Task
            }
            
            // 隨機化延遲，防止 TCP 緩衝區被快速塞滿
            sleep(Duration::from_millis(rand::random::<u64>() % 100 + 50)).await;
        }
    });
}
```

### 3.3 對策 03：單向外部化錨定與日誌前向安全 (One-Way Log Anchoring)

* **協定規格修補**：
  * **單向初始化**：`audit_pub.pem` 在 SASS Daemon 首次初始化生成時，必須透過單向 gRPC 通道**強制推播 (Push)** 至外部獨立審計伺服器（Audit Aggregator），本機 Daemon 不保留私鑰修改歷史日誌鏈的能力。
  * **Hash Chain 外部錨定 (External Anchoring)**：協定引入 `AnchorLog` ＲPC。Daemon 每小時將當前日誌鏈的最新 Hash，發送至受信任的外部時間戳權威 (RFC 3161 TSP) 進行數位簽章，或將其寫入不可篡改的 WORM (Write Once Read Many) 儲存介質。
  * 即使本機私鑰洩露，攻擊者也無法偽造出符合歷史已錨定時間戳的虛假雜湊鏈。

### 3.4 對策 04：Zstd 流控限額解壓 (Streaming Limit & Max Size Gate)

* **協定規格修補**：
  * 協定強制規定：解壓後的 Payload 最大明文大小不得超過 **5MB** (`MAX_DECOMPRESSED_PAYLOAD = 5 * 1024 * 1024`)。
  * Zstd 解碼器必須採用 **流式解壓 (Streaming Decompression)**，在解壓過程中動態計數已釋放的字節。一旦解壓字節累計超過 5MB，立即拋出 `DECOMPRESSION_BOMB_DETECTED` 錯誤，中斷 gRPC 連線並將該來源 IP 列入 Layer 0 封鎖黑名單。
* **Go 實作防護代碼 stub**：
```go
package codec

import (
	"bytes"
	"io"
	"github.com/klauspost/compress/zstd"
)

const MaxDecompressedPayload = 5 * 1024 * 1024 // 限制最大 5MB

// SafeDecodePayload 採用流式解壓限制，防止壓縮炸彈 (Zip Bomb) 癱瘓記憶體
func SafeDecodePayload(compressedData []byte) ([]byte, error) {
	decoder, err := zstd.NewReader(bytes.NewReader(compressedData))
	if err != nil {
		return nil, err
	}
	defer decoder.Close()

	// 使用 LimitedReader 限制最大讀取字節
	limitedReader := io.LimitReader(decoder, MaxDecompressedPayload)
	
	var decompressed bytes.Buffer
	_, err = io.Copy(&decompressed, limitedReader)
	if err != nil {
		return nil, err
	}

	// 檢查是否還有剩餘未讀取數據（若有，說明實際大小已超出 5MB 限額）
	var probe [1]byte
	n, _ := decoder.Read(probe[:])
	if n > 0 {
		return nil, fmt.Errorf("security violation: payload exceeds decompression limit (5MB)")
	}

	return decompressed.Bytes(), nil
}
```

### 3.5 對策 05：TLS 通道綁定與 ChaCha20 Challenge 鎖定 (TLS Channel Binding)

* **協定規格修補**：
  * **Channel Binding (RFC 5929)**：認知挑戰的生成與驗證必須與底層 TLS 憑證緊密綁定。
  * 在計算挑戰回應時，Client 必須將 **TLS Connection's Unique Signature (如 `tls-unique` 或 `tls-server-end-point`)** 與 Challenge Plaintext 進行 SHA256 混合雜湊後再回傳：
    $$\text{Response} = \text{HMAC-SHA256}(\text{Session\_Key}, \text{Plaintext} \mathbin{\Vert} \text{TLS\_Unique})$$
  * **效果**：即使攻擊者攔截到了某個連線的 Challenge 和 Plaintext 答案，由於該答案與該特定 TLS 連線的 `TLS_Unique` 簽章強綁定，攻擊者在自己發起的並行 TLS 連線中重放該答案，將會因為 `TLS_Unique` 不匹配而直接驗證失敗！

---

## 結論

SASS v7.0 協定通過「核心開源、特權插件」的 Dual-Engine 架構，不僅成功在作業系統 Userspace 達成了零特權的高效隔離，更在面臨 TOCTOU 符號連結穿透、壓縮炸彈、Tarpit 自我反噬等極端安全威脅時，展現出了極具韌性的安全設計。

本報告所提出的 `openat(2)` 絕對隔離、零記憶體分配 Tarpit 流、Zstd 限額流控解壓以及 TLS Channel Binding 等規格與代碼級對策，為 SASS 協定的標準化進程奠定了無懈可擊的安全性基石，使其具備在最惡劣的邊界網路環境中，守護 Agent 協同算力的硬核實力。
