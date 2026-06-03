# SakiAgentSSH Client — App Review & Testing Guide

> Version: 0.2.0

## App Overview

SakiAgentSSH Client is an **informational GUI companion** for the SakiAgentSSH gRPC-based remote execution client. This app displays About information, multilingual help documentation, and copyright details. The actual remote execution is provided by the CLI binary (`sakissh`).

## Test Environment

| Item | Requirement |
|------|-------------|
| OS | macOS 13.0 (Ventura) or later |
| Architecture | Apple Silicon (ARM64) |
| Network | Not required for launch |
| Sandbox | App Sandbox: Network Client |

## Testing Steps

### Step 1: Launch
1. Double-click `SakiAgentSSHClient.app`
2. **Expected**: About page appears with app name, version, feature list, download link, and copyright section with author avatar

### Step 2: Help (Multilingual)
1. Click the `?` button (top-right) or press `⌘?`
2. **Expected**: Help sheet opens with language picker (繁體中文 / English / 日本語)
3. Click each language tab
4. **Expected**: Content switches to the selected language

### Step 3: External Links
1. Click any link on the About page or Help sheet
2. **Expected**: Default browser opens the URL

### Step 4: Window Resizing
1. Resize the window
2. **Expected**: Content adapts, ScrollView works correctly

## Difference from Daemon

| Item | Daemon | Client |
|------|--------|--------|
| Network Entitlement | Client + Server | Client only |
| Purpose | Receive & execute remote commands | Send commands to remote Daemon |
| Default Port | 19284 (listen) | None (connect only) |

## Known Limitations

- This GUI app is **informational only** — it does not perform remote execution
- The actual client is provided by the CLI binary `sakissh`

## Privacy

- **No user data collected**
- No analytics or tracking
- No login or account required

## Contact

- Developer: Saki / Saki Studio
- Website: http://saki-studio.com.tw
- Email: Saki@saki-studio.com.tw
- GitHub: https://github.com/saki-tw
