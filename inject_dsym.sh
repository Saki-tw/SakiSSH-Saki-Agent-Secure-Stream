#!/bin/bash
# 將 Rust binary 的 dSYM 複製到最新的 Xcode archive 中
# 用法: 在 Xcode Archive 完成後立即執行

PROJ_DIR="$(cd "$(dirname "$0")" && pwd)"
ARCHIVE_DIR="$HOME/Library/Developer/Xcode/Archives/$(date +%Y-%m-%d)"

# 找最新的 Daemon archive
DAEMON_ARCHIVE=$(ls -td "$ARCHIVE_DIR"/SakiAgentSSHDaemon*.xcarchive 2>/dev/null | head -1)
if [ -n "$DAEMON_ARCHIVE" ]; then
    cp -r "$PROJ_DIR/SakiAgentSSH-Daemon/Resources/sakisshd.dSYM" "$DAEMON_ARCHIVE/dSYMs/" 2>/dev/null
    echo "✅ sakisshd.dSYM → $DAEMON_ARCHIVE/dSYMs/"
else
    echo "⚠️  找不到 Daemon archive"
fi

# 找最新的 Client archive
CLIENT_ARCHIVE=$(ls -td "$ARCHIVE_DIR"/SakiAgentSSHClient*.xcarchive 2>/dev/null | head -1)
if [ -n "$CLIENT_ARCHIVE" ]; then
    cp -r "$PROJ_DIR/SakiAgentSSH-Client/Resources/sakissh.dSYM" "$CLIENT_ARCHIVE/dSYMs/" 2>/dev/null
    echo "✅ sakissh.dSYM → $CLIENT_ARCHIVE/dSYMs/"
else
    echo "⚠️  找不到 Client archive"
fi
