# SakiAgentSSH Deployment Guide

## Configuration & Policy Directory
SakiAgentSSH Daemon and Client share a unified deployment structure.
All critical configurations and policies are stored in the user's home directory: `~/.sakissh/`

### 1. `config.json`
Generated automatically on first launch. Contains the daemon's binding IP, tokens, TLS configurations, and access control settings.

### 2. `13policy.yaml`
Generated automatically on first launch. Contains the rules for the 13Policy engine, including the list of dangerous commands and the `tarpit_size_mb` setting.
If an attacker attempts to run these commands, they will trigger the Tarpit countermeasure.

### 3. `chacha20.key`
Must be exactly 32 bytes and placed securely on both the daemon and client.
This symmetric key is used to generate the Cognitive Challenge during the authentication phase, ensuring the connected agent has genuine computational capabilities and is not a thin replay-attack wrapper.

## Deployment Steps
1. Transfer the compiled `sakisshd` and `sakissh` binaries to the target machine (e.g. U9-S700).
2. Start the daemon once to generate `~/.sakissh/config.json` and `~/.sakissh/13policy.yaml`.
3. (Optional) Adjust `13policy.yaml` based on local security requirements.
4. Securely copy `chacha20.key` to `~/.sakissh/chacha20.key` on the target machine.
5. Setup TLS certificates according to the paths defined in `config.json` if custom certificates are required.

This unified approach ensures compatibility across macOS, Linux, and Windows platforms (e.g., U9 environments).
