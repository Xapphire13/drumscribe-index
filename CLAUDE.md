# Drumscribe Index

A tool for browsing and exporting the Drumscribe drum transcription catalog. Fetches posts from Buy Me a Coffee, caches them locally, and exposes the index via a CLI and a native macOS SwiftUI app.

## Architecture

The project has two components:

- **`cli/`** — Rust CLI that fetches, caches, and outputs the song index
- **`ui/`** — macOS SwiftUI app that bundles the CLI binary and calls it as a subprocess

The app ships the CLI binary inside its bundle; `SongLoader.swift` executes it to load data and export files.

## Building

### CLI

```bash
cd cli
cargo build --release
# Output: cli/target/release/drumscribe-index
```

### macOS App

```bash
open ui/DrumscribeIndex.xcodeproj
# Then Product > Build, or Product > Archive for distribution
```

Requirements: macOS 14+, Xcode 15+.

## Running the CLI

```bash
# Initial fetch and cache
drumscribe-index

# Fetch new posts since last cache
drumscribe-index --update

# Export formats (--output is optional; defaults to stdout where applicable)
drumscribe-index --json [--output file.json]
drumscribe-index --markdown [--output file.md]
drumscribe-index --html [--output file.html]
drumscribe-index --xlsx --output file.xlsx
drumscribe-index --pdf --output file.pdf
```

Cache lives at `~/Library/Application Support/com.xapphire13.drumscribe-index/` on macOS.

## Key Source Files

| File | Role |
|------|------|
| `cli/src/main.rs` | CLI entry point, argument parsing (clap), song grouping |
| `cli/src/api/coffee_api.rs` | Buy Me a Coffee HTTP client (reqwest + async) |
| `cli/src/api/post.rs` | Serde models for API responses |
| `cli/src/models/song.rs` | Core types: `Song`, `SongGroup`, `Difficulty` |
| `cli/src/conversions/post.rs` | Post → Song parsing ("Title - Artist \| #N" heading format) |
| `cli/src/corrections.rs` | Artist name normalization |
| `cli/src/index_cache.rs` | Postcard binary cache read/write |
| `cli/src/output/` | One file per format: json, markdown, html, pdf, xlsx |
| `ui/DrumscribeIndex/SongLoader.swift` | Subprocess execution of bundled CLI |
| `ui/DrumscribeIndex/UpdateChecker.swift` | GitHub release checking and in-app update |
| `ui/DrumscribeIndex/FavoritesStore.swift` | UserDefaults-backed favorites |

## Release Process

```bash
# 1. Bump version (CLI bump cascades to UI)
./scripts/bump.sh --cli minor   # or --ui patch, --cli patch, etc.

# 2. Archive and export app from Xcode (Product > Archive > Direct Distribution)

# 3. Package
./scripts/make-dmg.sh "/path/to/Drumscribe Index.app"
shasum -a 256 "Drumscribe Index.dmg" > "Drumscribe Index.dmg.sha256"

# 4. Build CLI binaries (requires aarch64 + x86_64 targets)
./scripts/make-cli-release.sh --output-dir /path/to/release-dir

# 5. Create GitHub release with the DMG, DMG checksum, and CLI binaries/checksums
```

## Notable Details

- Only posts tagged with category `73044` (transcriptions) are indexed.
- Cache format is postcard (compact binary), not JSON.
- Pagination fetch is incremental: stops when it reaches already-cached content.
- Clippy pedantic lints are enabled — keep code clean.
- No test suite; validation is manual via CLI output and Xcode UI testing.
