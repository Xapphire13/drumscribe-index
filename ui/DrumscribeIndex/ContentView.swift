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
    @EnvironmentObject var favoritesStore: FavoritesStore
    @State private var exportFeedback: String?
    @State private var selectedDifficulties: Set<Difficulty> = []
    @State private var sortOption: SortOption = .titleAZ
    @State private var isGrouped: Bool = true
    @State private var showOnlyFavorites: Bool = false

    private func sortedSongs(_ songs: [Song]) -> [Song] {
        songs.sorted { a, b in
            switch sortOption {
            case .titleAZ:
                return a.title.localizedCaseInsensitiveCompare(b.title) == .orderedAscending
            case .recentlyAdded:
                return (Double(a.sequenceNumber) ?? 0) > (Double(b.sequenceNumber) ?? 0)
            case .difficultyAsc:
                return a.difficulty.sortOrder < b.difficulty.sortOrder
            case .difficultyDesc:
                if a.difficulty == .unrated { return false }
                if b.difficulty == .unrated { return true }
                return a.difficulty.sortOrder > b.difficulty.sortOrder
            }
        }
    }

    private var filteredSongs: [Song] {
        var result = groups.flatMap(\.songs)
        if !searchText.isEmpty {
            result = result.filter {
                $0.title.localizedCaseInsensitiveContains(searchText) ||
                $0.artist.localizedCaseInsensitiveContains(searchText)
            }
        }
        if !selectedDifficulties.isEmpty {
            result = result.filter { selectedDifficulties.contains($0.difficulty) }
        }
        if showOnlyFavorites {
            result = result.filter { favoritesStore.contains(id: $0.id) }
        }
        return result
    }

    private var displayGroups: [SongGroup] {
        let byArtist = Dictionary(grouping: filteredSongs, by: \.artist)
        return byArtist
            .map { SongGroup(artist: $0.key, songs: sortedSongs($0.value)) }
            .sorted { $0.artist < $1.artist }
    }

    private var displaySongs: [Song] {
        sortedSongs(filteredSongs)
    }

    private var subtitleText: String {
        let filtered = filteredSongs.count
        let total = groups.flatMap(\.songs).count
        if filtered == total {
            return "\(total) songs"
        } else {
            return "\(filtered) of \(total) songs"
        }
    }

    var body: some View {
        List {
            if isGrouped {
                ForEach(displayGroups) { group in
                    Section(group.artist) {
                        ForEach(group.songs) { song in
                            SongRow(song: song)
                        }
                    }
                }
            } else {
                ForEach(displaySongs) { song in
                    SongRow(song: song, showArtist: true)
                }
            }
        }
        .id("\(sortOption)-\(isGrouped)")
        .safeAreaInset(edge: .bottom, spacing: 0) {
            LastUpdatedBar(date: loader.lastUpdated, isUpdating: loader.isUpdating, exportFeedback: exportFeedback)
        }
        .searchable(text: $searchText, prompt: "Search songs or artists")
        .navigationTitle("Drumscribe Index")
        .navigationSubtitle(subtitleText)
        .frame(minWidth: 600, minHeight: 400)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    showOnlyFavorites.toggle()
                } label: {
                    Label("Favorites", systemImage: showOnlyFavorites ? "heart.fill" : "heart")
                }
                .help("Show only favorites")
            }

            ToolbarItem(placement: .primaryAction) {
                Menu {
                    ForEach(SortOption.allCases, id: \.self) { option in
                        Button {
                            sortOption = option
                        } label: {
                            Label(option.displayName, systemImage: sortOption == option ? "checkmark" : "")
                        }
                    }
                    Divider()
                    Button {
                        isGrouped.toggle()
                    } label: {
                        Label("Group by Artist", systemImage: isGrouped ? "checkmark" : "")
                    }
                } label: {
                    Label("Sort", systemImage: "arrow.up.arrow.down")
                }
                .help("Sort songs")
            }

            ToolbarItem(placement: .primaryAction) {
                Menu {
                    Button {
                        selectedDifficulties = []
                    } label: {
                        Label("All Difficulties", systemImage: selectedDifficulties.isEmpty ? "checkmark" : "")
                    }
                    Divider()
                    ForEach(Difficulty.allCases, id: \.self) { difficulty in
                        Button {
                            if selectedDifficulties.contains(difficulty) {
                                selectedDifficulties.remove(difficulty)
                            } else {
                                selectedDifficulties.insert(difficulty)
                            }
                        } label: {
                            Label(difficulty.rawValue, systemImage: selectedDifficulties.contains(difficulty) ? "checkmark" : "")
                        }
                    }
                } label: {
                    Label(
                        "Filter",
                        systemImage: selectedDifficulties.isEmpty ? "line.3.horizontal.decrease.circle" : "line.3.horizontal.decrease.circle.fill"
                    )
                }
                .help("Filter by difficulty")
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

            ToolbarItem(placement: .primaryAction) {
                Button {
                    loader.update()
                } label: {
                    Label("Update Index", systemImage: "arrow.clockwise")
                }
                .disabled(loader.isUpdating)
                .help("Fetch new songs from Drumscribe")
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
        case .pdf: .pdf
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
    var showArtist: Bool = false
    @EnvironmentObject var favoritesStore: FavoritesStore

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 1) {
                Text(song.title)
                if showArtist {
                    Text(song.artist)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            Spacer()
            DifficultyBadge(difficulty: song.difficulty)
            Button {
                favoritesStore.toggle(id: song.id)
            } label: {
                Image(systemName: favoritesStore.contains(id: song.id) ? "heart.fill" : "heart")
                    .foregroundStyle(favoritesStore.contains(id: song.id) ? .red : .secondary)
            }
            .buttonStyle(.borderless)
            .onHover { inside in
                if inside { NSCursor.pointingHand.push() } else { NSCursor.pop() }
            }
            Link(destination: URL(string: song.link)!) {
                Image(systemName: "arrow.up.right.square")
            }
            .buttonStyle(.borderless)
            .onHover { inside in
                if inside { NSCursor.pointingHand.push() } else { NSCursor.pop() }
            }
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
