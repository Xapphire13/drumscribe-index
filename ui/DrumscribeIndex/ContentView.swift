import SwiftUI

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
                SongListView(groups: groups, searchText: $searchText)
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
        .searchable(text: $searchText, prompt: "Search songs or artists")
        .navigationTitle("Drumscribe Index")
        .navigationSubtitle("\(groups.flatMap(\.songs).count) songs")
        .frame(minWidth: 600, minHeight: 400)
    }
}

struct SongRow: View {
    let song: Song

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 2) {
                Text(song.title)
                    .font(.body)
                Text(song.artist)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
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
