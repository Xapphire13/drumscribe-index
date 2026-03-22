import Foundation

enum Difficulty: String, Codable, CaseIterable {
    case beginner = "Beginner"
    case intermediate = "Intermediate"
    case advanced = "Advanced"
    case expert = "Expert"
    case master = "Master"
    case unrated = "Unrated"

    var sortOrder: Int {
        switch self {
        case .beginner:     1
        case .intermediate: 2
        case .advanced:     3
        case .expert:       4
        case .master:       5
        case .unrated:      6
        }
    }
}

enum SortOption: CaseIterable {
    case titleAZ
    case recentlyAdded
    case difficultyAsc
    case difficultyDesc

    var displayName: String {
        switch self {
        case .titleAZ:        "Title A–Z"
        case .recentlyAdded:  "Recently Added"
        case .difficultyAsc:  "Difficulty: Easy → Hard"
        case .difficultyDesc: "Difficulty: Hard → Easy"
        }
    }
}

struct Song: Identifiable, Codable {
    let id: Int
    let artist: String
    let title: String
    let difficulty: Difficulty
    let link: String
    let sequenceNumber: String
}

struct SongGroup: Identifiable, Decodable {
    var id: String { artist }
    let artist: String
    let songs: [Song]
}

extension [Song] {
    func grouped() -> [SongGroup] {
        let byArtist = Dictionary(grouping: self, by: \.artist)
        return byArtist
            .map { SongGroup(artist: $0.key, songs: $0.value.sorted { (Double($0.sequenceNumber) ?? 0) < (Double($1.sequenceNumber) ?? 0) }) }
            .sorted { $0.artist < $1.artist }
    }
}
