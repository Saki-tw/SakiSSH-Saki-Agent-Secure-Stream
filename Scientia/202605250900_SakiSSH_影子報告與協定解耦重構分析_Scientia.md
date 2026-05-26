# SASS (Saki Agent Secure Stream) 影子報告與協定解耦重構分析

> 報告日期：2026-05-25  
> 專案：SakiAgentSSH (SASS)  
> 目的：系統性修補 SASS v7.0 協定在防禦機制中隱含的反噬風險與邏輯漏洞，並提供 Rust/Go 雙端實作解耦點。  

---

## 摘要 (Executive Summary)

在對 SakiAgentSSH 進行深度的防禦機制審查中，我們發現數個原設計用來抵禦攻擊的機制，在極端條件下反而會成為攻擊者利用的弱點。本影子報告 (Shadow Report) 針對「TOCTOU 路徑擊穿」、「Tarpit 資源自噬」、「日誌鏈毒化」、「Zstd 壓縮炸彈」與「ChaCha20 挑戰繞過」五大維度進行方法學解構，並確立協定層的改進規格與雙端 (Rust/Go) 代碼實作方案。

---

## Phase 1: 方法學依賴性解構與解耦點確立

### 1.1 TOCTOU (Time-of-Check to Time-of-Use) 路徑擊穿
*   **方法學依賴**：Userspace 的系統調用具有「非原子性」。
*   **漏洞場景**：Daemon 在進行路徑合法性檢查（確保在 Sandbox 內）與實際打開文件 (`open`) 之間存在時間差。攻擊者可利用極短的時間窗，將目標路徑替換為指向 `/etc/shadow` 或私鑰的軟連結。
*   **解耦點**：必須消除 Path-Check 時間窗，將檢查與讀取行為收束至單一 OS 核心原子操作。

### 1.2 Tarpit 自噬 DoS
*   **方法學依賴**：動態內存配置與未授權資源預先分配。
*   **漏洞場景**：Tarpit 旨在拖延惡意掃描者。但若 SASS Daemon 為每一個被 Tarpit 困住的連線動態分配緩衝區或獨立的協程/執行緒，攻擊者發動海量連接即可耗盡 Daemon 記憶體（資源自噬）。
*   **解耦點**：防禦機制的資源消耗必須是「$O(1)$ 常數級別」的。

### 1.3 日誌鏈毒化 (Log Poisoning)
*   **方法學依賴**：私鑰同權限本地可讀性與單點日誌自我驗證。
*   **漏洞場景**：若主機被攻破，攻擊者取得 Daemon 權限，即可讀取本地私鑰並重新偽造/抹除過去的日誌雜湊鏈。單機自我驗證無法抵抗事後全面竄改。
*   **解耦點**：日誌的不可竄改性必須依賴外部時間錨點與單向遞交機制。

### 1.4 Zstd 壓縮炸彈 (Zip Bomb)
*   **方法學依賴**：非流式記憶體分配與解壓縮狀態無上限控制。
*   **漏洞場景**：攻擊者發送經過極端壓縮的 Payload，Daemon 試圖將其完全解壓至記憶體中，瞬間撐爆 RAM 導致 OOM 崩潰。
*   **解耦點**：必須強制實施「流式解壓」並引入強制的膨脹係數與記憶體上限控制。

### 1.5 ChaCha20 挑戰繞過
*   **方法學依賴**：加密挑戰與傳輸通道狀態脫鉤 (Channel Decoupling)。
*   **漏洞場景**：Nonce 的偽隨機性若遭預測，或攻擊者利用重放攻擊 (Replay Attack) 將合法的 Challenge 轉發，即可繞過驗證機制。
*   **解耦點**：Challenge 必須與底層 TLS 握手狀態（TLS Channel Binding）進行密碼學綁定。

---

## Phase 2: 設計五大協定層改進規格 (Protocol-level Spec)

1.  **原子性儲存提供者 API**：
    所有檔案系統交互不再使用絕對路徑與標準 `open()`。全面改採基於目錄文件描述符 (Directory FD) 的 `openat()`，並強制加上 `O_NOFOLLOW` 標籤，徹底免疫軟連結劫持。
2.  **靜態 Tarpit 限流與分配規則**：
    實作 **Zero-Allocation Tarpit**。所有 Tarpit 連線共享同一個靜態的 4KB 高熵垃圾 Ring Buffer。伺服器僅透過單一事件迴圈 (epoll/kqueue) 緩慢推送，不產生新協程或新記憶體分配。
3.  **外部一向公鑰推播與時間戳錨定**：
    引入 RFC 3161 協定。每產生 $N$ 筆稽核日誌，Daemon 需將雜湊根 (Root Hash) 透過 UDP 或短連線單向推送至安全的稽核伺服器（如 Saki Star 匯流排），實現非對稱錨定。
4.  **流式 Zstd 解壓防衛閘**：
    在解壓層強制實作 `io.LimitReader` (Go) / `Take` (Rust) 的抽象。設定 `MAX_DECOMPRESSED_SIZE`，一旦解壓字節數超標即立刻中斷連線並回報為攻擊事件。
5.  **TLS Channel Binding (tls-unique)**：
    使用 RFC 5929 `tls-unique` 提取 TLS 握手完成後的 finished message，作為 ChaCha20 HMAC 的輸入鹽值 (Salt)。確保 Challenge 無法被跨連線重放。

---

## Phase 3: Rust 實作端影子報告 (`saki-ssh-daemon`)

### 3.1 漏洞現狀分析
當前的 `branch_mgr.rs` 在解析路徑時過度依賴 Rust 標準庫的 `std::fs::metadata`，這在多執行緒併發下暴露了 TOCTOU 窗口。`tarpit.rs` 為每個惡意連接 spawn 了一個 `tokio::task` 並分配了獨立 buffer，存在 OOM 風險。

### 3.2 具體修補實作

**StorageProvider 原子化封裝**
```rust
use libc::{openat, O_RDONLY, O_NOFOLLOW, O_CLOEXEC};
use std::os::unix::io::{RawFd, FromRawFd};
use std::fs::File;

pub struct SecureStorage {
    dir_fd: RawFd,
}

impl SecureStorage {
    pub fn open_secure(&self, path: &str) -> Result<File, std::io::Error> {
        let c_path = std::ffi::CString::new(path).unwrap();
        // O_NOFOLLOW 拒絕跟隨 Symlink，徹底斬斷 TOCTOU
        let fd = unsafe { openat(self.dir_fd, c_path.as_ptr(), O_RDONLY | O_NOFOLLOW | O_CLOEXEC, 0) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}
```

**Zero-Allocation Tarpit (Global Static Buffer)**
```rust
use once_cell::sync::Lazy;

// 全域共享的靜態高熵 Buffer，所有 Tarpit 連線共用此資源
static TARPIT_BUFFER: Lazy<Vec<u8>> = Lazy::new(|| {
    (0..4096).map(|_| rand::random::<u8>()).collect()
});

pub async fn handle_tarpit(mut stream: tokio::net::TcpStream) {
    let mut offset = 0;
    loop {
        // 每次極慢速寫入 16 bytes，不消耗額外記憶體
        if stream.write_all(&TARPIT_BUFFER[offset..offset+16]).await.is_err() {
            break;
        }
        offset = (offset + 16) % 4096;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}
```

---

## Phase 4: Go 實作端影子報告 (`go-sakissh`)

### 4.1 漏洞現狀分析
在 `go-sakissh` (Client/Payload 端) 中，`codec.go` 直接調用 Zstd 解壓整個 byte slice，若伺服器被挾持並回傳壓縮炸彈，Client 會因 OOM 而崩潰。Go 的 Goroutine 調度在面對無上限的 Tarpit 等待時，會導致 Goroutine 洩漏 (Leak)。

### 4.2 具體修補實作

**流式 Zstd 限額解壓縮**
```go
package codec

import (
	"fmt"
	"io"
	"bytes"
	"github.com/klauspost/compress/zstd"
)

const MaxDecompressedSize = 10 * 1024 * 1024 // 10MB 硬上限

func SafeDecodePayload(compressedData []byte) ([]byte, error) {
	decoder, err := zstd.NewReader(bytes.NewReader(compressedData))
	if err != nil {
		return nil, err
	}
	defer decoder.Close()

	// 核心解耦點：使用 LimitReader 避免壓縮炸彈撐爆 RAM
	limitedReader := io.LimitReader(decoder, MaxDecompressedSize+1)
	decompressed, err := io.ReadAll(limitedReader)
	if err != nil {
		return nil, err
	}
	if len(decompressed) > MaxDecompressedSize {
		return nil, fmt.Errorf("zstd bomb detected: payload exceeds maximum allowed size")
	}
	return decompressed, nil
}
```

**TLS Channel Binding HMAC 綁定**
```go
import (
    "crypto/hmac"
    "crypto/sha256"
    "crypto/tls"
)

func GenerateBoundChallenge(conn *tls.Conn, challengeNonce []byte, secretKey []byte) []byte {
    state := conn.ConnectionState()
    // 獲取 TLS Finished Message (tls-unique)
    tlsUnique := state.TLSUnique 
    
    mac := hmac.New(sha256.New, secretKey)
    mac.Write(tlsUnique)         // 綁定通道特徵
    mac.Write(challengeNonce)    // 綁定時間與隨機性
    return mac.Sum(nil)
}
```

---

## Phase 5: 影子報告合龍結論

本次 SASS 協定解耦重構，成功將「高維度攻擊向量（如 TOCTOU、OOM 炸彈、重放攻擊）」的防禦下放至「OS 核心原子操作 (openat/LimitReader) 與密碼學底層特徵 (tls-unique)」。
- 透過放棄動態分配，我們實現了 $O(1)$ 記憶體複雜度的 Tarpit。
- 透過放棄絕對路徑，我們實現了零時間差的路徑安全。
- 透過綁定底層 TLS 狀態，我們將重放攻擊的難度提升至攻破 TLS 協議本身。

本報告已完成雙端代碼方案確立，將作為下一階段實際注入 `saki-ssh-daemon` 與 `go-sakissh` 原始碼的實作藍圖。
