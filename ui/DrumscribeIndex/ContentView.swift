import SwiftUI

private let placeholderGroups: [SongGroup] = [
    SongGroup(artist: "Foo Fighters", songs: [
        Song(id: 1, artist: "Foo Fighters", title: "Everlong", difficulty: .advanced, link: "https://example.com/1", sequenceNumber: "1"),
        Song(id: 2, artist: "Foo Fighters", title: "Best of You", difficulty: .intermediate, link: "https://example.com/2", sequenceNumber: "2"),
    ]),
    SongGroup(artist: "John Bonham", songs: [
        Song(id: 3, artist: "John Bonham", title: "Moby Dick", difficulty: .master, link: "https://example.com/3", sequenceNumber: "3"),
    ]),
    SongGroup(artist: "The Police", songs: [
        Song(id: 4, artist: "The Police", title: "Roxanne", difficulty: .beginner, link: "https://example.com/4", sequenceNumber: "4"),
    ]),
]

struct ContentView: View {
    @State private var searchText = ""
    @State private var groups: [SongGroup] = placeholderGroups

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
        Text(difficulty.rawValue)
            .font(.caption2)
            .fontWeight(.medium)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(color.opacity(0.15), in: Capsule())
            .foregroundStyle(color)
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
