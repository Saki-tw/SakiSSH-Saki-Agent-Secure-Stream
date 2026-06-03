# Scoop-SakiStudio

> **Saki Studio's Scoop Bucket** — because sometimes the main repository isn't ready for what's coming.

## Why This Bucket Exists

This bucket exists because ScoopInstaller/Main rejected Vi for not being "famous enough" — despite all automated checks passing ✅.

Meanwhile, Microsoft Winget accepted it on first submission.

### Scoop maintainer's exact words / 原文

> I suggest maintaining this in your personal bucket instead. Please feel free to reopen this PR once it fully meets the criteria.
>
> — z-Fng, ScoopInstaller/Main #7643

---

## Available Packages

| Package | Description | Version |
|---------|-------------|---------|
| `saki-vi` | Vi — the original Vi, not Vim. Trilingual (繁中 / 日本語 / EN) | 1.0.0 |
| `sakiagentssh-daemon` | SakiAgentSSH Daemon — Agent-native gRPC execution daemon | 0.2.0 |
| `sakiagentssh-client` | SakiAgentSSH Client — Agent-native gRPC execution CLI | 0.2.0 |
| `sakiclip` | SakiClip — Cross-machine clipboard sync | *coming soon* |

## Usage

```powershell
# Add bucket
scoop bucket add sakistudio https://github.com/Saki-tw/Scoop-SakiStudio

# Install
scoop install saki-vi
scoop install sakiagentssh-daemon
scoop install sakiagentssh-client

# Update
scoop update *
```

## Bucket Structure

```
Scoop-SakiStudio/
└── bucket/
    ├── saki-vi.json
    ├── sakiagentssh-daemon.json
    └── sakiagentssh-client.json
```

## Migration from scoop-sakivi

```powershell
scoop bucket rm sakivi
scoop bucket add sakistudio https://github.com/Saki-tw/Scoop-SakiStudio
```

GitHub auto-redirects from the old URL, but updating your local bucket is recommended.

---

**Saki Studio** · [saki-studio.com.tw](http://saki-studio.com.tw) · [GitHub](https://github.com/saki-tw)
