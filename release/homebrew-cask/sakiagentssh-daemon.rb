cask "sakiagentssh-daemon" do
  version "0.2.0"
  sha256 "f655ff2a35cb18ba81da0d083f1b6f7f293184592d067aed87723928118fe8ee"

  url "https://github.com/saki-tw/SakiAgentSSH/releases/download/v#{version}/SakiAgentSSHDaemon.dmg"
  name "SakiAgentSSH Daemon"
  desc "Agent-native cross-machine execution daemon over gRPC"
  homepage "https://github.com/saki-tw/SakiAgentSSH"

  depends_on macos: ">= :ventura"

  app "SakiAgentSSHDaemon.app"

  zap trash: [
    "~/Library/Caches/tw.com.saki-studio.SakiAgentSSH-Daemon-GUIapp",
    "~/Library/Preferences/tw.com.saki-studio.SakiAgentSSH-Daemon-GUIapp.plist",
  ]
end
