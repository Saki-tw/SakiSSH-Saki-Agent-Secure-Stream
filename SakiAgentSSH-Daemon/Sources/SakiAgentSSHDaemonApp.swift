import SwiftUI

@main
struct SakiAgentSSHDaemonApp: App {
    var body: some Scene {
        WindowGroup {
            AboutView()
        }
        .defaultSize(width: 520, height: 520)
        .commands {
            CommandGroup(replacing: .help) {
                Button("SakiAgentSSH 說明書") {
                    NotificationCenter.default.post(name: .showHelp, object: nil)
                }
                .keyboardShortcut("?", modifiers: .command)
            }
        }
    }
}

extension Notification.Name {
    static let showHelp = Notification.Name("showHelp")
}

// MARK: - Custom Font
extension Font {
    static func saki(_ size: CGFloat) -> Font {
        .custom("GenJyuuGothicX-Regular", size: size)
    }
}

// MARK: - Saki Studio Colors
extension Color {
    static let sakiPurple = Color(red: 218/255, green: 112/255, blue: 214/255)
    static let sakiBlue   = Color(red: 0/255, green: 206/255, blue: 209/255)
}

// MARK: - Help Locale
enum HelpLocale: String, CaseIterable {
    case zhHant = "zh-Hant"
    case enUS = "en-US"
    case jaJP = "ja-JP"

    var displayName: String {
        switch self {
        case .zhHant: return "繁體中文"
        case .enUS: return "English"
        case .jaJP: return "日本語"
        }
    }

    var filename: String { "help_\(rawValue)" }

    static func detect() -> HelpLocale {
        let lang = Locale.current.language.languageCode?.identifier ?? "en"
        switch lang {
        case "zh": return .zhHant
        case "ja": return .jaJP
        default: return .enUS
        }
    }
}

// MARK: - Help View
struct HelpView: View {
    @State private var selectedLocale = HelpLocale.detect()
    @State private var helpContent: String = ""

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("SakiAgentSSH Help")
                    .font(.saki(16))
                    .fontWeight(.bold)
                    .foregroundStyle(
                        LinearGradient(colors: [.sakiPurple, .sakiBlue],
                                     startPoint: .leading, endPoint: .trailing)
                    )
                Spacer()
                Picker("", selection: $selectedLocale) {
                    ForEach(HelpLocale.allCases, id: \.self) { locale in
                        Text(locale.displayName).tag(locale)
                    }
                }
                .pickerStyle(.segmented)
                .frame(width: 280)
            }
            .padding(.horizontal, 20)
            .padding(.vertical, 12)

            Divider()

            ScrollView {
                Text(helpContent)
                    .font(.saki(13))
                    .lineSpacing(6)
                    .padding(20)
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .frame(minWidth: 500, minHeight: 450)
        .background(
            LinearGradient(colors: [
                Color.sakiPurple.opacity(0.02),
                Color.sakiBlue.opacity(0.02)
            ], startPoint: .topLeading, endPoint: .bottomTrailing)
        )
        .onAppear { loadHelp() }
        .onChange(of: selectedLocale) { _ in loadHelp() }
    }

    private func loadHelp() {
        if let url = Bundle.main.url(forResource: selectedLocale.filename, withExtension: "md"),
           let content = try? String(contentsOf: url, encoding: .utf8) {
            helpContent = content
        } else {
            helpContent = "Help file not found."
        }
    }
}

// MARK: - About View
struct AboutView: View {
    @State private var showHelp = false
    private let appVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0.2.0"
    private let buildNumber = Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "1"

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("About SakiAgentSSH Daemon")
                    .font(.saki(14))
                    .fontWeight(.semibold)
                    .foregroundStyle(.secondary)
                Spacer()
                Button { showHelp = true } label: {
                    Image(systemName: "questionmark.circle")
                        .font(.system(size: 16))
                        .foregroundStyle(Color.sakiBlue)
                }
                .buttonStyle(.plain)
                .help("說明書 (⌘?)")
            }
            .padding(.horizontal, 20)
            .padding(.top, 16)
            .padding(.bottom, 12)

            Divider()

            ScrollView {
                VStack(spacing: 20) {
                    Spacer(minLength: 16)

                    if let appIcon = NSImage(named: "AppIcon") {
                        Image(nsImage: appIcon)
                            .resizable()
                            .aspectRatio(contentMode: .fit)
                            .frame(width: 128, height: 128)
                            .shadow(color: .sakiPurple.opacity(0.3), radius: 12, y: 6)
                    }

                    VStack(spacing: 6) {
                        Text("SakiAgentSSH Daemon")
                            .font(.saki(22))
                            .fontWeight(.bold)
                            .foregroundStyle(
                                LinearGradient(colors: [.sakiPurple, .sakiBlue],
                                             startPoint: .leading, endPoint: .trailing)
                            )
                        Text("v\(appVersion) (Build \(buildNumber))")
                            .font(.saki(12))
                            .foregroundStyle(.secondary)
                    }

                    Text("Agent 原生跨機器執行協議\n基於 gRPC 的安全遠端指令服務")
                        .font(.saki(13))
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.center)

                    Divider().padding(.horizontal, 40)

                    VStack(alignment: .leading, spacing: 8) {
                        FeatureRow(icon: "shield.checkered", text: "CIDR 白名單安全存取控制", color: .sakiPurple)
                        FeatureRow(icon: "network", text: "gRPC 雙向串流通訊", color: .sakiBlue)
                        FeatureRow(icon: "desktopcomputer", text: "跨平台支援 macOS / Windows", color: .sakiPurple)
                        FeatureRow(icon: "terminal", text: "Agent 原生指令代理", color: .sakiBlue)
                    }
                    .padding(.horizontal, 32)

                    Divider().padding(.horizontal, 40)

                    VStack(spacing: 4) {
                        Text("Windows 版本下載")
                            .font(.saki(12))
                            .foregroundStyle(.secondary)
                        Link("sakisshd.exe (GitHub Release)",
                             destination: URL(string: "https://github.com/saki-tw/SakiAgentSSH/releases/download/v0.2.0/sakisshd.exe")!)
                            .font(.saki(11))
                    }

                    Divider().padding(.horizontal, 40)
                    CopyrightView()
                    Spacer(minLength: 16)
                }
            }
        }
        .frame(minWidth: 440, minHeight: 400)
        .background(
            LinearGradient(colors: [
                Color.sakiPurple.opacity(0.03),
                Color.sakiBlue.opacity(0.03)
            ], startPoint: .topLeading, endPoint: .bottomTrailing)
        )
        .sheet(isPresented: $showHelp) {
            HelpView()
                .frame(width: 600, height: 650)
        }
        .onReceive(NotificationCenter.default.publisher(for: .showHelp)) { _ in
            showHelp = true
        }
    }
}

struct FeatureRow: View {
    let icon: String
    let text: String
    let color: Color
    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: icon)
                .font(.system(size: 14))
                .foregroundStyle(color)
                .frame(width: 20)
            Text(text)
                .font(.saki(12))
                .foregroundStyle(.primary)
        }
    }
}

struct CopyrightView: View {
    var body: some View {
        VStack(spacing: 4) {
            Text("© 2026 Saki Studio. All rights reserved.")
                .font(.saki(11))
                .foregroundStyle(.tertiary)
            HStack(spacing: 12) {
                Link("saki-studio.com.tw", destination: URL(string: "http://saki-studio.com.tw")!)
                    .font(.saki(10))
                Link("GitHub", destination: URL(string: "https://github.com/saki-tw")!)
                    .font(.saki(10))
            }
            Text("Saki@saki-studio.com.tw")
                .font(.saki(10))
                .foregroundStyle(.tertiary)
        }
    }
}
