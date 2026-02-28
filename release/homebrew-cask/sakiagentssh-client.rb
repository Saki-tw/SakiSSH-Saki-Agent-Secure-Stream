cask "sakiagentssh-client" do
  version "0.2.0"
  sha256 "6b873363dfa782709fb2f4f3eb120e7b010602a57f5b127e80cf237207ce76d5"

  url "https://github.com/saki-tw/SakiAgentSSH/releases/download/v#{version}/SakiAgentSSHClient.dmg"
  name "SakiAgentSSH Client"
  desc "Agent-native remote execution CLI over gRPC"
  homepage "https://github.com/saki-tw/SakiAgentSSH"

  depends_on macos: ">= :ventura"

  app "SakiAgentSSHClient.app"

  zap trash: [
    "~/Library/Caches/tw.com.saki-studio.SakiAgentSSH-Client-GUIApp",
    "~/Library/Preferences/tw.com.saki-studio.SakiAgentSSH-Client-GUIApp.plist",
  ]
end
