import SwiftUI
import UniformTypeIdentifiers

struct ContentView: View {
    @StateObject private var loader = SongLoader()
    @State private var searchText = ""

    var body: some View {
        Group {
            switch loader.state {
            case .loading:
                ProgressView("Loading songs…")
                    .frame(minWidth: 600, minHeight: 400)
            case .loaded(let groups):
                SongListView(groups: groups, searchText: $searchText, loader: loader)
            case .failed(let msg):
                VStack(spacing: 12) {
                    Text("Failed to load songs: \(msg)")
                        .foregroundStyle(.secondary)
                    Button("Retry") { loader.load() }
                }
                .frame(minWidth: 600, minHeight: 400)
            }
        }
        .onAppear { loader.load() }
    }
}

private struct SongListView: View {
    let groups: [SongGroup]
    @Binding var searchText: String
    @ObservedObject var loader: SongLoader
    @State private var exportFeedback: String?

    private var filteredGroups: [SongGroup] {
        guard !searchText.isEmpty else { return groups }
        return groups.compactMap { group in
            let matchingArtist = group.artist.localizedCaseInsensitiveContains(searchText)
            let matchingSongs = group.songs.filter {
                $0.title.localizedCaseInsensitiveContains(searchText) ||
                $0.artist.localizedCaseInsensitiveContains(searchText)
            }
            if matchingArtist {
                return group
            } else if !matchingSongs.isEmpty {
                return SongGroup(artist: group.artist, songs: matchingSongs)
            }
            return nil
        }
    }

    var body: some View {
        List {
            ForEach(filteredGroups) { group in
                Section(group.artist) {
                    ForEach(group.songs) { song in
                        SongRow(song: song)
                    }
                }
            }
        }
        .safeAreaInset(edge: .bottom, spacing: 0) {
            LastUpdatedBar(date: loader.lastUpdated, isUpdating: loader.isUpdating, exportFeedback: exportFeedback)
        }
        .searchable(text: $searchText, prompt: "Search songs or artists")
        .navigationTitle("Drumscribe Index")
        .navigationSubtitle("\(groups.flatMap(\.songs).count) songs")
        .frame(minWidth: 600, minHeight: 400)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    loader.update()
                } label: {
                    Label("Update Index", systemImage: "arrow.clockwise")
                }
                .disabled(loader.isUpdating)
                .help("Fetch new songs from Drumscribe")
            }

            ToolbarItem(placement: .primaryAction) {
                Menu {
                    ForEach(ExportFormat.allCases) { format in
                        Button(format.displayName) {
                            showExportPanel(format: format)
                        }
                    }
                } label: {
                    Label("Export", systemImage: "square.and.arrow.up")
                }
                .help("Export the song index")
            }
        }
    }

    private func showExportPanel(format: ExportFormat) {
        let panel = NSSavePanel()
        panel.allowedContentTypes = [contentType(for: format)]
        panel.nameFieldStringValue = "drumscribe-index.\(format.fileExtension)"
        panel.isExtensionHidden = false
        panel.begin { response in
            guard response == .OK, let url = panel.url else { return }
            Task {
                try? await loader.export(format: format, to: url)
                exportFeedback = url.lastPathComponent
                try? await Task.sleep(for: .seconds(3))
                exportFeedback = nil
            }
        }
    }

    private func contentType(for format: ExportFormat) -> UTType {
        switch format {
        case .json: .json
        case .markdown: UTType(filenameExtension: "md") ?? .data
        case .html: .html
        case .xlsx: UTType(filenameExtension: "xlsx") ?? .data
        }
    }
}

private struct LastUpdatedBar: View {
    let date: Date?
    let isUpdating: Bool
    let exportFeedback: String?

    private var label: String {
        guard let date else { return "Never updated" }
        return "Last updated: \(date.formatted(date: .abbreviated, time: .shortened))"
    }

    var body: some View {
        VStack(spacing: 0) {
            Divider()
            HStack(spacing: 5) {
                if isUpdating {
                    ProgressView()
                        .controlSize(.mini)
                    Text("Updating…")
                } else if let exportFeedback {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundStyle(.green)
                    Text("Exported \(exportFeedback)")
                } else {
                    Text(label)
                }
            }
            .font(.caption)
            .foregroundStyle(.secondary)
            .frame(maxWidth: .infinity, alignment: .center)
            .padding(.vertical, 5)
            .background(.bar)
        }
    }
}

struct SongRow: View {
    let song: Song

    var body: some View {
        HStack {
            Text(song.title)
            Spacer()
            DifficultyBadge(difficulty: song.difficulty)
            Link(destination: URL(string: song.link)!) {
                Image(systemName: "arrow.up.right.square")
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
        }
        .padding(.vertical, 2)
    }
}

struct DifficultyBadge: View {
    let difficulty: Difficulty

    var body: some View {
        if let starCount {
            HStack(spacing: 2) {
                ForEach(0..<starCount, id: \.self) { _ in
                    Image(systemName: "star.fill")
                        .font(.caption2)
                }
            }
            .foregroundStyle(color)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(color.opacity(0.15), in: Capsule())
        }
    }

    private var starCount: Int? {
        switch difficulty {
        case .beginner: 1
        case .intermediate: 2
        case .advanced: 3
        case .expert: 4
        case .master: 5
        case .unrated: nil
        }
    }

    private var color: Color {
        switch difficulty {
        case .beginner: .green
        case .intermediate: .blue
        case .advanced: .orange
        case .expert: .red
        case .master: .purple
        case .unrated: .gray
        }
    }
}

#Preview {
    NavigationStack {
        ContentView()
    }
}
