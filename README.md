# Drumscribe Index

A tool for browsing and exporting the
[Drumscribe](https://buymeacoffee.com/drumscribe) drum transcription catalog. It
fetches transcription posts, caches them locally, and makes the index available
via a CLI (JSON, Markdown, HTML, XLSX, PDF output) and an optional native macOS
app.

## Prerequisites

**Rust** is required for both the CLI and the macOS app (the Xcode build
invokes Cargo internally):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Visit [rustup.rs](https://rustup.rs) for platform-specific instructions.

**Xcode 15+ and macOS 14 (Sonoma) or later** — required only if you want the
macOS app.

## CLI

### Install

#### Option 1: Install with Cargo (recommended)

```bash
cargo install --git https://github.com/Xapphire13/drumscribe-index.git --path cli
```

Installs `drumscribe-index` to your Cargo bin directory (`~/.cargo/bin/`),
making it available system-wide.

#### Option 2: Build from source

```bash
git clone https://github.com/Xapphire13/drumscribe-index.git
cd drumscribe-index/cli
cargo build --release
```

The compiled binary will be at `cli/target/release/drumscribe-index`.

### Usage

On first run, the program fetches all available songs and caches them locally.
Subsequent runs use the cache. Use `--update` to fetch new songs:

```bash
drumscribe-index --update
```

#### Output formats

| Flag | Output | Notes |
|------|--------|-------|
| _(none / `--json`)_ | JSON | Default |
| `--markdown` | Markdown | |
| `--html` | HTML | |
| `--xlsx` | Excel | Requires `--output` |
| `--pdf` | PDF | Requires `--output` |

Use `--output <file>` to save to a file instead of stdout:

```bash
drumscribe-index --markdown --output index.md
drumscribe-index --html --output index.html
drumscribe-index --json --output songs.json
drumscribe-index --xlsx --output songs.xlsx
drumscribe-index --pdf --output songs.pdf
drumscribe-index --update --xlsx --output songs.xlsx
```

## macOS App (optional)

The native SwiftUI app lives in `ui/`. It is **optional** — the CLI works
independently without it. Building the app also builds and bundles the CLI, so
you do not need a separate CLI install if you are using the app.

### Build

Open in Xcode:

```bash
open ui/DrumscribeIndex.xcodeproj
```

Or build from the command line:

```bash
cd ui && xcodebuild -scheme DrumscribeIndex build
```

### Distribution (DMG)

1. **Export the app from Xcode**: `Product > Archive`, then in the Organizer
   select the archive and click **Distribute App > Direct Distribution**.
   Xcode will produce a `DrumscribeIndex.app`.

2. **Install `create-dmg`** (one-time):

    ```bash
    brew install create-dmg
    ```

3. **Run the DMG script**:

    ```bash
    ./scripts/make-dmg.sh /path/to/DrumscribeIndex.app
    ```

   This creates `DrumscribeIndex.dmg` in the same directory as the `.app`.

> **Note:** The app is unsigned, so recipients will need to right-click > Open
> on first launch, or allow it in System Settings > Privacy & Security.

## Output Structure

The index organizes songs by artist, with each song containing:

- **Artist name**
- **Song title**
- **Difficulty level** (Beginner, Intermediate, Advanced, Expert, Master, or
  Unrated)
- **Link** to the full transcription on Drumscribe
- **Sequence number** (YouTube/PDF song identifier)

## Cache Location

The program stores cached data in your system's standard application data
directory:

- **macOS**: `~/Library/Application Support/com.xapphire13.drumscribe-index/`
- **Linux**: `~/.local/share/drumscribe-index/`
- **Windows**:
  `C:\Users\<YourUsername>\AppData\Roaming\xapphire13\drumscribe-index\`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file
for details.
