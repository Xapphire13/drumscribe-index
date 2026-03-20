import Foundation

@MainActor
final class SongLoader: ObservableObject {
    enum State {
        case loading
        case loaded([SongGroup])
        case failed(String)
    }

    @Published var state: State = .loading

    private let cliBinaryURL = Bundle.main.executableURL!
        .deletingLastPathComponent()
        .appendingPathComponent("drumscribe-index")

    func load() {
        state = .loading
        Task {
            do {
                let groups = try await runCLI()
                state = .loaded(groups)
            } catch {
                state = .failed(error.localizedDescription)
            }
        }
    }

    private func runCLI() async throws -> [SongGroup] {
        return try await Task.detached(priority: .userInitiated) {
            let process = Process()
            process.executableURL = self.cliBinaryURL
            process.arguments = ["--json"]

            let pipe = Pipe()
            process.standardOutput = pipe
            process.standardError = Pipe() // discard stderr

            try process.run()
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            process.waitUntilExit()

            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            return try decoder.decode([SongGroup].self, from: data)
        }.value
    }
}
