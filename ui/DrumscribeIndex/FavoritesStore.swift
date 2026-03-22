import Foundation

@MainActor
final class FavoritesStore: ObservableObject {
    @Published private(set) var favoriteIDs: Set<Int> = []

    private var saveURL: URL? {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)
            .first?
            .appending(components: "com.xapphire13.drumscribe-index", "favorites.json")
    }

    init() {
        load()
    }

    func toggle(id: Int) {
        if favoriteIDs.contains(id) {
            favoriteIDs.remove(id)
        } else {
            favoriteIDs.insert(id)
        }
        save()
    }

    func contains(id: Int) -> Bool {
        favoriteIDs.contains(id)
    }

    private func load() {
        guard let url = saveURL,
              let data = try? Data(contentsOf: url),
              let ids = try? JSONDecoder().decode(Set<Int>.self, from: data) else { return }
        favoriteIDs = ids
    }

    private func save() {
        guard let url = saveURL,
              let data = try? JSONEncoder().encode(favoriteIDs) else { return }
        try? data.write(to: url, options: .atomic)
    }
}
