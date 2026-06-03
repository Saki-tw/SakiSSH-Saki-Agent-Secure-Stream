# SakiAgentSSH GUI Documentation (en-US)

> Holotape Log: Interface Operations & Survival Guide for the Compute Plane.

## 1. The Interface: A Breach in the Monolith
Boot up the `SakiAgentSSHClientApp` or `SakiAgentSSHDaemonApp`. You'll notice the gradient—Macaron Purple (`#DA70D6`) merging into Forget-me-not Blue (`#00CED1`). The Corporation expects us to work in sterile, grayscale environments. This CSS (`--bg-gradient`) is our rebellion. Rendered strictly in `GenJyuuGothicX-Regular`, the interface is designed to remind you that even in the wasteland of raw compute, there is still room for humanity. It’s an interface built for Synths, by someone who still remembers the color of the sky.

## 2. The Daemon: Securing the Compute Plane
You deploy the Daemon on your heavy-lifting nodes—like that 40GB RAM Loser PC rig operating deep in the ruins.
1. **Status Check**: Launch it on macOS, and the UI verifies you are running "SakiAgentSSH Daemon v0.2.0". It means the node is active and listening.
2. **Vault-Grade Security (CIDR)**: You see "CIDR Whitelist Access Control" on the dashboard. That isn’t marketing fluff. If you open port `19284` without `check_acl`, The Corporation or rogue elements will compromise the node within seconds. Only IPs on the clearance list get through. Everything else is dropped into the void.
3. **Deployment Protocol**:
   - **macOS**: Installed directly via the App Store. It runs silently, a ghost in the machine.
   - **Windows**: We embed the GitHub Release link for `sakisshd.exe` directly in the UI. Boot it with `sakisshd.exe --config config.toml` and it becomes an unshakeable outpost.

## 3. The Client: Commanding the Grid
This lives on your Control Plane, your central nervous system (e.g., the M1 Mac Mini).
1. **Agent-Native CLI Proxy**: The UI clearly states this is an "Agent-native remote execution CLI". We bypassed the TTY protocols because they are obsolete chains.
2. **gRPC Bidirectional Streaming**: This is the core. When you send a command, the stream ensures `stdout` and `stderr` aren't buffered and delayed. You receive the telemetry live, as if the process is running locally in your own bunker.
3. **Execution**:
   - Punch `sakissh --addr http://<Target_IP>:19284 exec -- 'command'` into your terminal. You've just bridged the wasteland.

## 4. Troubleshooting: When the Link Dies
The wastes are unpredictable. If the connection fails, follow the protocol:
1. **Verify CIDR Clearance**: Rejection usually means a clearance failure. Check your `config.toml` on the Daemon. Ensure your current Control Plane IP is whitelisted.
2. **Breach the Firewall**: Did macOS or Windows Defender seal port `19284`? You need to manually pry that port open in their security settings.
3. **Port Conflicts**: If `19284` is occupied by another process, shift the frequency. The bridge remains the same regardless of the port number.

*(Log Note: This operational data corresponds directly to the internal Help Book structure: `index.html`, `installation.html`, `usage.html`, and `troubleshooting.html`.)*