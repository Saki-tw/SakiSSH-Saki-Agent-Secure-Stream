# SakiAgentSSH 跨平台套件管理上架研究

> **建立時間**：20260228_0528 (UTC+8)
> **標籤**：#SakiAgentSSH #上架 #Homebrew #Winget #研究

---

## 一、Homebrew 上架策略

### Formula vs Cask 選擇

| 類型 | 適用場景 | SakiAgentSSH 適用？ |
|------|---------|:---:|
| **Formula** | CLI 工具、可從原始碼編譯 | ✅ |
| **Cask** | GUI 應用、預編譯二進位（.dmg/.pkg） | ❌ |

**結論**：SakiAgentSSH 為 CLI 工具（daemon + client），應使用 **Formula**。

### Formula 上架規格

1. **GitHub 公開倉庫**：`https://github.com/saki-tw/SakiAgentSSH`
2. **GitHub Release 發佈**：附帶 tarball（`.tar.gz`）
3. **SHA256 校驗碼**：每個 release tarball 需計算
4. **Notability 門檻**：至少 30 forks/watchers 或 75 stars
   - ⚠️ **目前可能不符合門檻** → 先使用私有 Tap

### 推薦策略：私有 Tap（短期）→ 官方 Formula（長期）

#### 短期：自建 Homebrew Tap
```bash
# 建立 Tap repo: github.com/saki-tw/homebrew-tools
brew tap saki-tw/tools
brew install saki-tw/tools/sakisshd
brew install saki-tw/tools/sakissh
```

#### Formula 範例結構
```ruby
class Sakisshd < Formula
  desc "SakiAgentSSH Daemon — Agent-native cross-machine execution"
  homepage "https://github.com/saki-tw/SakiAgentSSH"
  url "https://github.com/saki-tw/SakiAgentSSH/releases/download/v0.2.0/sakisshd-darwin-arm64.tar.gz"
  sha256 "SHA256_HASH_HERE"
  license "LicenseRef-SakiStudio"
  version "0.2.0"

  def install
    bin.install "sakisshd"
  end

  service do
    run [opt_bin/"sakisshd"]
    keep_alive true
    working_dir var/"sakisshd"
    log_path var/"log/sakisshd.log"
    error_log_path var/"log/sakisshd.error.log"
  end
end
```

### 檔案分離策略（單平台下載）
```
releases/
├── v0.2.0/
│   ├── sakisshd-darwin-arm64.tar.gz     # macOS daemon
│   ├── sakisshd-darwin-arm64.tar.gz.sha256
│   ├── sakissh-darwin-arm64.tar.gz      # macOS client
│   ├── sakissh-darwin-arm64.tar.gz.sha256
│   ├── sakisshd-windows-x86_64.zip     # Windows daemon
│   ├── sakisshd-windows-x86_64.zip.sha256
│   ├── sakissh-windows-x86_64.zip      # Windows client
│   └── sakissh-windows-x86_64.zip.sha256
```

---

## 二、Winget 上架策略

### Manifest 格式要求

Winget 使用 YAML manifest，提交至 `microsoft/winget-pkgs` GitHub：

#### 目錄結構
```
manifests/
  s/
    SakiStudio/
      SakiAgentSSH/
        0.2.0/
          SakiStudio.SakiAgentSSH.yaml          # 版本檔
          SakiStudio.SakiAgentSSH.installer.yaml # 安裝器檔
          SakiStudio.SakiAgentSSH.locale.en-US.yaml
          SakiStudio.SakiAgentSSH.locale.zh-TW.yaml
```

#### Manifest 範例
```yaml
# SakiStudio.SakiAgentSSH.yaml
PackageIdentifier: SakiStudio.SakiAgentSSH
PackageVersion: 0.2.0
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
```

```yaml
# SakiStudio.SakiAgentSSH.installer.yaml
PackageIdentifier: SakiStudio.SakiAgentSSH
PackageVersion: 0.2.0
InstallerType: exe
Installers:
  - Architecture: x64
    InstallerUrl: https://github.com/saki-tw/SakiAgentSSH/releases/download/v0.2.0/sakisshd-setup.exe
    InstallerSha256: SHA256_HASH_HERE
    InstallerSwitches:
      Silent: /S
      SilentWithProgress: /S
ManifestType: installer
ManifestVersion: 1.6.0
```

### Winget 限制與應對
| 限制 | 說明 | SakiAgentSSH 應對 |
|------|------|-----------------|
| 必須靜默安裝 | 安裝器不得顯示對話框 | install.ps1 已支援 `-Silent` |
| 必須支援 exe/msi/msix | 不支援 script-based | 考慮製作 Inno Setup 安裝器 |
| 安全審查 | 自動掃描病毒 | 確保 release 無誤報 |
| 活躍度要求 | 較新項目可能被拒 | 先以 GitHub Release 為主 |

### 建議順序
1. **短期**：GitHub Release 直接下載（已可執行）
2. **中期**：自建 Homebrew Tap（`saki-tw/homebrew-tools`）
3. **長期**：Winget 提交（需製作 exe 安裝器）+ Homebrew Core 申請（需知名度）

---

## 三、版本號與自動升級策略

### 語意化版本 (SemVer)
```
v{MAJOR}.{MINOR}.{PATCH}
例：v0.2.0 → v0.2.1 (bugfix) → v0.3.0 (ED25519 認證)
```

### 版本判定基準
| 變更類型 | 版本遞增 | 範例 |
|---------|---------|------|
| 安全修補 | PATCH | 修補 CIDR 白名單繞過 |
| 新功能 | MINOR | 加入 ED25519 認證 |
| 破壞性變更 | MAJOR | config.json 格式不相容 |

### 自動升級機制
- **Homebrew**：`brew upgrade sakisshd`（Tap 自動追蹤最新版）
- **Winget**：`winget upgrade SakiStudio.SakiAgentSSH`
- **GitHub Release**：Release Notes 內建 CHANGELOG

---

## 四、跨平台上架 SOP 總結

### 發佈流程
1. 更新 `Cargo.toml` 版本號
2. `cargo build --release` 雙平台（macOS ARM64 + Windows x86_64）
3. Strip + 計算 SHA256
4. 建立 GitHub Release + 上傳分平台 tarball/zip
5. 更新 Homebrew Tap Formula
6. （若適用）提交 Winget manifest PR
