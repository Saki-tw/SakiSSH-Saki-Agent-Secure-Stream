# Homebrew Cask PR — Notability Evidence

## 提交 Checklist

When submitting to `homebrew/homebrew-cask`, include the following notability evidence in the PR description:

---

### PR Template

```markdown
## App: SakiAgentSSH (Daemon / Client)

Agent-native cross-machine execution protocol over gRPC. Replaces SSH's TTY-bound model with bidirectional streaming for AI agent integration.

### Notability Evidence

1. **Official Website**: [saki-studio.com.tw](https://saki-studio.com.tw) (Hugo on Cloudflare)

2. **App Store** (same developer, Team ID: 36HPTNN8NU):
   - [SakiAgentSkills](https://apps.apple.com/tw/app/saki-agent-skills/id6758680481?mt=12) — Agent skill orchestration
   - [SakiMCP](https://apps.apple.com/tw/app/sakimcp/id6758668850?mt=12) — Model Context Protocol server

3. **Winget** (same developer, accepted on first submission):
   - [SakiStudio.SakiVi on winstall.app](https://winstall.app/apps/SakiStudio.SakiVi)
   - Vi editor for Windows — trilingual (繁中/EN/日本語)

4. **SakiAgentSSH App Store**: Pending review (same developer account)

### Technical Details
- MIT Licensed
- Signed with Apple Developer ID (36HPTNN8NU)
- App Sandbox enabled
- No external dependencies
```

---

### Winget 連結證明

| 來源 | URL | Status |
|------|-----|--------|
| winstall.app | https://winstall.app/apps/SakiStudio.SakiVi | ✅ 200 OK |
| winget.run | https://winget.run/pkg/SakiStudio/SakiVi | ⚠️ 500 (site issue) |
| GitHub manifests | https://github.com/microsoft/winget-pkgs/tree/master/manifests/s/SakiStudio/SakiVi | ⚠️ 404 (path may vary) |

> **推薦**: 使用 `winstall.app` 作為 Winget 上架證明
