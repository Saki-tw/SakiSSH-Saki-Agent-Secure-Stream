# App Store Review 回覆 — Guideline 2.4.5(i) com.apple.security.network.server

> **Submission ID**: f2b070ce-7b8d-4302-ba9a-f0d997a2fb62
> **Review Date**: March 02, 2026
> **Response Date**: March 03, 2026
> **Version Reviewed**: 0.2

---

## Response

Hello,

Thank you for reviewing SakiAgentSSH Daemon and for your question regarding the `com.apple.security.network.server` entitlement. We'd like to provide a detailed explanation of how and why this entitlement is required.

### What SakiAgentSSH Daemon Is

SakiAgentSSH Daemon is a **developer tool** that enables AI coding assistants (such as Google Gemini CLI and Anthropic Claude) to execute shell commands and transfer files on a remote machine over the network, using the gRPC protocol.

The app belongs to the category `public.app-category.developer-tools` and is functionally analogous to other remote execution or SSH-style utilities available on macOS — but designed specifically for AI-driven automation workflows rather than interactive human terminal sessions.

### Why `com.apple.security.network.server` Is Required

The core function of this application is to **accept incoming gRPC connections from remote AI agent clients**. This is a server-side role by definition:

1. **Listening for incoming connections**: The daemon binds to a configurable TCP port (default: `19284`) and listens for gRPC/HTTP2 connections from authorized clients on the local network or via VPN (e.g., Tailscale).

2. **CIDR-based access control**: The daemon enforces a whitelist of allowed client IP ranges (`allowed_cidrs` in its configuration) before accepting any connection. Unauthorized connections are immediately rejected with `PERMISSION_DENIED`.

3. **Streaming execution**: Once authenticated, the daemon executes requested commands locally and streams `stdout`/`stderr` back to the client in real-time via bidirectional gRPC streaming.

4. **File transfer**: The daemon accepts file uploads and serves file downloads using chunked gRPC streaming, supporting resume via byte offset.

Without the `com.apple.security.network.server` entitlement, the application **cannot** perform its primary function of accepting incoming network connections from remote clients.

### How to Observe This Functionality

The application is a **CLI tool** — its core functionality operates via the command-line binary (`sakisshd`) which is bundled with the app. The GUI window serves as the companion interface providing:
- Application About information and version details
- Multilingual help documentation (繁體中文, English, 日本語)
- CLI usage instructions and download links for cross-platform clients

To test the server functionality:
1. Launch the app (the daemon service starts and listens on the configured port)
2. From another machine on the same network, run: `sakissh --host <daemon-ip>:19284 ping`
3. The daemon will respond with its version, uptime, and OS information

### Comparable Apps on the Mac App Store

This type of tool is analogous to several categories of apps that legitimately use `com.apple.security.network.server`:
- **SSH server/daemon utilities** that accept incoming remote connections
- **Remote development tools** that allow IDE clients to connect and execute code remotely
- **Network service applications** (e.g., local web servers, file sharing tools) that listen for incoming connections

### Summary of Entitlements Used

| Entitlement | Purpose |
|-------------|---------|
| `com.apple.security.app-sandbox` | Required for App Store distribution |
| `com.apple.security.network.client` | Opening external URLs (GitHub, website) from within the app |
| `com.apple.security.network.server` | **Primary function**: Accepting incoming gRPC connections from remote AI agent clients |

We hope this clarifies the necessity of the `com.apple.security.network.server` entitlement. SakiAgentSSH Daemon is fundamentally a server application — accepting incoming connections is its core function, not a peripheral feature.

Please let us know if you need any additional information or a demonstration.

Best regards,
Saki
Saki Studio
Saki@saki-studio.com.tw
https://saki-studio.com.tw
