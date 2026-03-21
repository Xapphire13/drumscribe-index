import Foundation

enum ExportFormat: String, CaseIterable, Identifiable {
    case pdf, xlsx, html, markdown, json
    var id: String { rawValue }
    var cliFlag: String {
        switch self {
        case .pdf: "--pdf"
        case .json: "--json"
        case .markdown: "--markdown"
        case .html: "--html"
        case .xlsx: "--xlsx"
        }
    }
    var fileExtension: String {
        switch self {
        case .pdf: "pdf"
        case .json: "json"
        case .markdown: "md"
        case .html: "html"
        case .xlsx: "xlsx"
        }
    }
    var displayName: String {
        switch self {
        case .pdf: "PDF"
        case .json: "JSON"
        case .markdown: "Markdown"
        case .html: "HTML"
        case .xlsx: "Excel (XLSX)"
        }
    }
}

@MainActor
final class SongLoader: ObservableObject {
    enum State {
        case loading
        case loaded([SongGroup])
        case failed(String)
    }

    @Published var state: State = .loading
    @Published var isUpdating: Bool = false
    @Published var lastUpdated: Date? = nil

    private let cliBinaryURL = Bundle.main.executableURL!
        .deletingLastPathComponent()
        .appendingPathComponent("drumscribe-index")

    func load() {
        state = .loading
        Task {
            do {
                let groups = try await runCLI()
                state = .loaded(groups)
                loadLastUpdated()
            } catch {
                state = .failed(error.localizedDescription)
                loadLastUpdated()
            }
        }
    }

    func update() {
        isUpdating = true
        Task {
            defer { isUpdating = false }
            do {
                try await runCLIRaw(arguments: ["--update"])
                let groups = try await runCLI()
                state = .loaded(groups)
                loadLastUpdated()
            } catch {
                // Update failed silently; state remains loaded
            }
        }
    }

    private var indexFileURL: URL? {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)
            .first?
            .appending(components: "com.xapphire13.drumscribe-index", "index.bin")
    }

    private func loadLastUpdated() {
        guard let url = indexFileURL,
              let attrs = try? FileManager.default.attributesOfItem(atPath: url.path),
              let date = attrs[.modificationDate] as? Date else {
            lastUpdated = nil
            return
        }
        lastUpdated = date
    }

    func export(format: ExportFormat, to url: URL) async throws {
        try await runCLIRaw(arguments: [format.cliFlag, "--output", url.path])
    }

    private func runCLI() async throws -> [SongGroup] {
        return try await Task.detached(priority: .userInitiated) {
            let process = Process()
            process.executableURL = self.cliBinaryURL
            process.arguments = ["--json"]

            let pipe = Pipe()
            process.standardOutput = pipe
            process.standardError = FileHandle.nullDevice

            try process.run()
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            process.waitUntilExit()

            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            return try decoder.decode([SongGroup].self, from: data)
        }.value
    }

    private func runCLIRaw(arguments: [String]) async throws {
        try await Task.detached(priority: .userInitiated) {
            let process = Process()
            process.executableURL = self.cliBinaryURL
            process.arguments = arguments
            process.standardOutput = FileHandle.nullDevice
            process.standardError = FileHandle.nullDevice
            try process.run()
            process.waitUntilExit()
            guard process.terminationStatus == 0 else {
                throw NSError(domain: "CLI", code: Int(process.terminationStatus))
            }
        }.value
    }
}
