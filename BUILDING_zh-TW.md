# SakiAgentSSH 建置指南 (Build Guide)

[🇹🇼 繁體中文](BUILDING_zh-TW.md) | [🇯🇵 日本語](BUILDING_ja.md) | [🇺🇸 English](BUILDING.md)

這是一份寫在編譯過程裡的指南。就像在廢墟中重新點燃一座反應爐，我們需要依序準備好工具鏈，然後看著那些 `*.rs` 檔案在 `cargo` 的催化下，變成守護我們算力孤島的神經索。

## 準備工具鏈 (Prerequisites)

在開始編譯之前，請確保你的機器上已經裝備了以下工具：

1.  **Rust Toolchain** (1.75+)
    - 這是一切的基礎。透過 `rustup` 安裝是最優雅的方式。
    - Linux / macOS: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
    - Windows: 請前往 [rustup.rs](https://rustup.rs/) 下載 `rustup-init.exe`。
2.  **Protocol Buffers Compiler (`protoc`)**
    - `tonic` 框架依賴它來解析 `proto/sakissh.proto`，將契約化為程式碼。
    - macOS: `brew install protobuf`
    - Windows: `choco install protoc` 或是透過 Scoop 安裝。
    - Ubuntu/Debian: `apt install -y protobuf-compiler libprotobuf-dev`
3.  **macOS GUI 專屬依賴**
    - 若你需要編譯帶有馬卡龍紫與勿忘草青介面的 macOS App，你需要：
    - Xcode 16+
    - XcodeGen: `brew install xcodegen`

## 編譯核心 CLI 程式 (Rust Core)

這部分產出的純命令列工具，不帶任何 GUI 負擔，最適合直接部署在無頭 (Headless) 伺服器上。

```bash
# 首先，進入專案的根目錄
cd SakiAgentSSH

# 編譯 Daemon (守護進程，部署於運算端)
cd saki-ssh-daemon
cargo build --release
# 編譯完成後，你可以在 target/release/sakisshd 找到它

# 退回上層，接著編譯 Client (客戶端，部署於控制端)
cd ../saki-ssh-client
cargo build --release
# 產出物位於 target/release/sakissh
```

### Profile 設定說明

在 `Cargo.toml` 中，我們針對 `[profile.release]` 做了極致的優化：
- `strip = "symbols"`：拋棄那些沉重的 Debug 符號，讓二進位檔更輕盈。
- `lto = true`：開啟 Link Time Optimization，榨乾每一滴效能。
- `opt-level = "z"`：針對檔案大小進行優化。

## 編譯 macOS GUI 應用程式 (SwiftUI)

我們為 SakiAgentSSH 包裝了優雅的 SwiftUI 介面。要編譯它們，我們使用 `xcodegen` 來動態生成 `.xcodeproj` 專案檔。

```bash
# 確保你已經在專案根目錄

# --------------------------
# 1. 編譯 Daemon App
# --------------------------
cd SakiAgentSSH-Daemon
# 生成 Xcode 專案
xcodegen generate
# 透過命令列進行 Release 編譯
xcodebuild build -configuration Release -scheme SakiAgentSSHDaemon
# 編譯完成的 .app 會出現在 build/Release/ 目錄下

# --------------------------
# 2. 編譯 Client App
# --------------------------
cd ../SakiAgentSSH-Client
xcodegen generate
xcodebuild build -configuration Release -scheme SakiAgentSSHClient
```

> **注意**：我們極度建議不要把 `.xcodeproj` 檔案加入 Git 追蹤（已經寫入 `.gitignore`），請永遠依賴 `project.yml` 透過 `xcodegen` 來生成，這樣能避免各種令人抓狂的合併衝突。

## 跨平台編譯 (Cross-Compilation)

如果你需要在一台機器上（例如你的 M1 Mac）為另一台機器（例如 Loser PC 的 Windows）編譯：

```bash
# 新增目標平台的 target
rustup target add x86_64-pc-windows-msvc

# 安裝跨平台編譯工具 (例如 cargo-zigbuild 或設定對應的 linker)
# 接著執行：
cargo build --release --target x86_64-pc-windows-msvc
```

但老實說，跨平台編譯常常會遇到 C++ 依賴的問題。在我們的架構中，如果需要 Windows 版本，直接丟給 Windows 節點去編譯通常是更乾脆的選擇。

## 故障排除 (Troubleshooting)

- **`protoc` 找不到**：如果你在 `cargo build` 時看到關於 `tonic-build` 或 `prost` 的錯誤，99% 是因為你的系統環境變數（PATH）裡沒有 `protoc`。請重新確認安裝。
- **macOS App 閃退或無法打開**：如果是手動複製 `.app`，請確保移除了隔離屬性：`xattr -dr com.apple.quarantine /Applications/SakiAgentSSHDaemon.app`。

-- 
*「程式碼聞起來像暫存器操作的氣味，這就是編譯的聲音。」*
