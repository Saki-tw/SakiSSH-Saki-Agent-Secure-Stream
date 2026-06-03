# Building SakiAgentSSH from Source

## Prerequisites

### CLI (Rust) — All Platforms

| 工具 | 版本 | 安裝方式 |
|------|------|---------|
| Rust toolchain | ≥ 1.75 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| protoc | ≥ 3.21 | macOS: `brew install protobuf` / Windows: `choco install protoc` |

### macOS GUI Apps (SwiftUI)

| 工具 | 版本 | 安裝方式 |
|------|------|---------|
| Xcode | ≥ 16.0 | Mac App Store |
| XcodeGen | latest | `brew install xcodegen` |
| macOS SDK | ≥ 13.0 | Bundled with Xcode |

### Windows Cross-Compile (from macOS)

```bash
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64
```

---

## Build CLI Binaries

### macOS (ARM64)

```bash
# Daemon
cd saki-ssh-daemon
cargo build --release
# Output: target/release/sakisshd

# Client
cd saki-ssh-client
cargo build --release
# Output: target/release/sakissh
```

### Windows (x86_64, cross-compile from macOS)

```bash
# Daemon
cd saki-ssh-daemon
cargo build --release --target x86_64-pc-windows-gnu
# Output: target/x86_64-pc-windows-gnu/release/sakisshd.exe

# Client
cd saki-ssh-client
cargo build --release --target x86_64-pc-windows-gnu
# Output: target/x86_64-pc-windows-gnu/release/sakissh.exe
```

### Windows (native build)

```powershell
# 需要 Visual Studio Build Tools 或 GNU toolchain
cd saki-ssh-daemon
cargo build --release
# Output: target\release\sakisshd.exe
```

---

## Build macOS GUI Apps (.app)

GUI Apps 是 SwiftUI wrapper，提供：
- 「關於」頁面（icon、版本號、功能列表）
- 三語系說明書（繁中 / EN / 日本語），Cmd+? 開啟
- Windows 版本下載連結
- 色彩主題（馬卡龍紫 `#DA70D6` / 勿忘草青 `#00CED1`）
- 自訂字型 GenJyuuGothicX-Regular

> **Note**: GUI Apps 不包含 Rust CLI binary。它們是獨立的 SwiftUI 應用程式，用於展示專案資訊和提供說明文件。CLI binary 需另外安裝。

### Build Steps

```bash
# 1. Generate Xcode project
cd SakiAgentSSH-Daemon
xcodegen generate

# 2. Build
xcodebuild build -project SakiAgentSSHDaemon.xcodeproj \
    -scheme SakiAgentSSHDaemon \
    -configuration Release \
    SYMROOT=$(pwd)/build_out \
    ONLY_ACTIVE_ARCH=NO

# 3. App is at: build_out/Release/SakiAgentSSHDaemon.app
```

對 Client 重複同樣步驟（替換 Daemon → Client）。

### Archive for App Store

```bash
# 必須先移除 quarantine
xattr -dr com.apple.quarantine .

# Archive（建議在 Xcode GUI 中執行 Product → Archive）
xcodebuild clean -project SakiAgentSSHDaemon.xcodeproj -scheme SakiAgentSSHDaemon
xcodebuild -project SakiAgentSSHDaemon.xcodeproj \
    -scheme SakiAgentSSHDaemon \
    -configuration Release \
    archive -archivePath ./build/SakiAgentSSHDaemon.xcarchive
```

### DMG for Direct Distribution

```bash
hdiutil create -volname "SakiAgentSSH Daemon" \
    -srcfolder build_out/Release/SakiAgentSSHDaemon.app \
    -ov -format UDZO \
    SakiAgentSSHDaemon.dmg
```

---

## Project Dependencies

### Rust Crates (CLI)

| Crate | Purpose |
|-------|---------|
| `tonic` | gRPC server/client |
| `prost` | Protocol Buffers |
| `tokio` | Async runtime |
| `ipnetwork` | CIDR ACL |
| `serde` / `serde_json` | Config parsing |
| `clap` | CLI argument parsing |

### Swift Frameworks (GUI)

| Framework | Purpose |
|-----------|---------|
| SwiftUI | User interface |
| Combine | Reactive event handling (Help menu) |

---

## Release Checklist

```
[ ] cargo build --release (macOS ARM64)
[ ] cargo build --release --target x86_64-pc-windows-gnu (Windows)
[ ] strip binaries
[ ] xcodegen generate (Daemon + Client)
[ ] xcodebuild archive (or Xcode GUI)
[ ] hdiutil create DMG
[ ] shasum -a 256 *.dmg *.exe
[ ] Update Scoop manifest hash
[ ] Update Homebrew Cask hash
[ ] Update Winget manifest hash
[ ] git tag vX.Y.Z
[ ] Upload to GitHub Releases
```
