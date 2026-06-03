# Winget Submission Guide

## Manifests

SakiAgentSSH 的 Winget manifests 遵循 [winget-pkgs schema v1.6.0](https://github.com/microsoft/winget-pkgs)。

### Daemon

```
manifests/s/SakiStudio/SakiAgentSSH/Daemon/0.2.0/
├── SakiStudio.SakiAgentSSH.Daemon.yaml
├── SakiStudio.SakiAgentSSH.Daemon.installer.yaml
├── SakiStudio.SakiAgentSSH.Daemon.locale.en-US.yaml
└── SakiStudio.SakiAgentSSH.Daemon.locale.zh-Hant.yaml
```

### Client

```
manifests/s/SakiStudio/SakiAgentSSH/Client/0.2.0/
├── SakiStudio.SakiAgentSSH.Client.yaml
├── SakiStudio.SakiAgentSSH.Client.installer.yaml
└── SakiStudio.SakiAgentSSH.Client.locale.en-US.yaml
```

## Submission Steps

```bash
# 1. Fork microsoft/winget-pkgs
# 2. 複製 manifests 到正確路徑
mkdir -p manifests/s/SakiStudio/SakiAgentSSH/Daemon/0.2.0/
mkdir -p manifests/s/SakiStudio/SakiAgentSSH/Client/0.2.0/
cp release/winget/SakiStudio.SakiAgentSSH.Daemon.* manifests/s/SakiStudio/SakiAgentSSH/Daemon/0.2.0/
cp release/winget/SakiStudio.SakiAgentSSH.Client.* manifests/s/SakiStudio/SakiAgentSSH/Client/0.2.0/

# 3. 本地驗證
winget validate manifests/s/SakiStudio/SakiAgentSSH/Daemon/0.2.0/
winget validate manifests/s/SakiStudio/SakiAgentSSH/Client/0.2.0/

# 4. 建 PR 到 microsoft/winget-pkgs main branch
```

## Prior Submissions

- ✅ `SakiStudio.SakiVi` — Accepted on first submission
  - Repo: [Saki-tw/VI-SakiWin64](https://github.com/Saki-tw/VI-SakiWin64)
