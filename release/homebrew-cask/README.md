# Homebrew Cask — SakiAgentSSH

## Notability

- **Official Website**: [saki-studio.com.tw](https://saki-studio.com.tw)
- **App Store** (同開發者):
  - [SakiAgentSkills](https://apps.apple.com/tw/app/saki-agent-skills/id6758680481?mt=12)
  - [SakiMCP](https://apps.apple.com/tw/app/sakimcp/id6758668850?mt=12)
- **Winget** (同開發者): `SakiStudio.SakiVi` — accepted on first submission

## 使用方式

```bash
# 加入 Tap
brew tap saki-tw/tools https://github.com/saki-tw/homebrew-tools

# 安裝
brew install --cask sakiagentssh-daemon
brew install --cask sakiagentssh-client

# 移除
brew uninstall --cask sakiagentssh-daemon
brew uninstall --cask sakiagentssh-client
```

## Tap 設置步驟

1. 建立 GitHub repo `saki-tw/homebrew-tools`（若尚未存在）
2. 將 `Casks/` 目錄放入 repo：
   ```
   homebrew-tools/
   └── Casks/
       ├── sakiagentssh-daemon.rb
       └── sakiagentssh-client.rb
   ```
3. 在 `saki-tw/SakiAgentSSH` 的 GitHub Release v0.2.0 上傳：
   - `SakiAgentSSHDaemon.dmg`
   - `SakiAgentSSHClient.dmg`
   - `sakisshd.exe`（Windows Daemon）
   - `sakissh.exe`（Windows Client）

## SHA256

| 檔案 | SHA256 |
|------|--------|
| SakiAgentSSHDaemon.dmg | `d15e00bf14a222392603cf8c8b118f1c97977920e187df1b5d8419a4dae6b94e` |
| SakiAgentSSHClient.dmg | `3c064bb2e9ae6260dd45a37d1a101b14e050a7120911ca3ae84753ee9a3ee01f` |

## 版本更新 SOP

1. Archive 新版 → DMG 打包
2. `shasum -a 256 *.dmg`
3. 更新 .rb 中的 `version` + `sha256`
4. 上傳 DMG 至 GitHub Release
5. `git push homebrew-tools`
