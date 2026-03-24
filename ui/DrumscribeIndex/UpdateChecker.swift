import AppKit
import Foundation

@MainActor
final class UpdateChecker: ObservableObject {
    struct UpdateInfo {
        let version: String
    }

    @Published var availableUpdate: UpdateInfo? = nil
    @Published var isChecking: Bool = false
    @Published var isDismissed: Bool = false

    private var currentVersion: (Int, Int, Int) {
        let v = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0.0.0"
        return Self.parseSemver(v)
    }

    func check() async {
        isChecking = true
        isDismissed = false
        defer { isChecking = false }
        await fetchAndCompare()
    }

    func checkOnStartup() async {
        isChecking = true
        defer { isChecking = false }
        await fetchAndCompare()
    }

    func dismiss() {
        isDismissed = true
    }

    func installUpdate() {
        let pid = ProcessInfo.processInfo.processIdentifier
        let cmd = "curl -fsSL https://raw.githubusercontent.com/Xapphire13/drumscribe-index/master/scripts/install.sh | bash -s -- --wait-pid \(pid) --open"
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/osascript")
        process.arguments = ["-e", "tell application \"Terminal\" to do script \"\(cmd)\""]
        try? process.run()
        NSApp.terminate(nil)
    }

    private func fetchAndCompare() async {
        guard let url = URL(string: "https://api.github.com/repos/Xapphire13/drumscribe-index/releases/latest") else { return }
        guard let (data, _) = try? await URLSession.shared.data(from: url) else { return }
        guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let tagName = json["tag_name"] as? String else { return }

        // tag format: "ui/v0.2.0"
        let versionString = tagName
            .replacingOccurrences(of: "ui/v", with: "")
            .replacingOccurrences(of: "v", with: "")
        let remote = Self.parseSemver(versionString)

        if remote > currentVersion {
            availableUpdate = UpdateInfo(version: versionString)
        }
    }

    private static func parseSemver(_ v: String) -> (Int, Int, Int) {
        let parts = v.split(separator: ".").compactMap { Int($0) }
        return (parts.indices.contains(0) ? parts[0] : 0,
                parts.indices.contains(1) ? parts[1] : 0,
                parts.indices.contains(2) ? parts[2] : 0)
    }
}
