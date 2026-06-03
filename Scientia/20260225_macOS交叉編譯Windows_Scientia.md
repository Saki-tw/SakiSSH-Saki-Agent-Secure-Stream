# macOS M1 交叉編譯 Windows Rust 二進位檔研究 (cargo-xwin)

> **建立時間**：2026-02-25 19:55 (UTC+8)
> **標籤**：#交叉編譯 #Rust #Windows #M1

## 核心問題
Windows 節點 (Loser/Trading PC) 資源有限，不適合安裝龐大的 Visual Studio Build Tools。且目標機器可能缺少 `link.exe`，導致無法在本機編譯 `saki-ssh-daemon`。

## 解決方案：cargo-xwin
在 macOS M1 (aarch64-apple-darwin) 上使用 `cargo-xwin` 進行交叉編譯，目標為 `x86_64-pc-windows-msvc`。

### 優勢
- **免安裝 VS**: `cargo-xwin` 會自動下載所需的 Windows SDK 與 CRT headers/libs。
- **效能**: 利用 M1 的高效能進行編譯，直接交付成品至目標機器。
- **純淨**: 目標節點只需執行環境，不需安裝開發工具鏈。

### 執行步驟
1. 安裝工具：`cargo install cargo-xwin`
2. 加入目標：`rustup target add x86_64-pc-windows-msvc`
3. 編譯：`cargo xwin build --release --target x86_64-pc-windows-msvc`

## 實踐結果
成功編譯出 `sakisshd.exe` (2.2MB)，具備 gRPC 功能。此方法證實了 Saki Studio 「極限生存開發」哲學的可行性。
