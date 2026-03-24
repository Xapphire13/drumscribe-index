# Drumscribe Index

A tool for browsing and exporting the
[Drumscribe](https://buymeacoffee.com/drumscribe) drum transcription catalog. It
fetches transcription posts, caches them locally, and makes the index available
via a CLI (JSON, Markdown, HTML, XLSX, PDF output) and an optional native macOS
app.

## Prerequisites

**Rust** — required only if you install via Cargo or build from source (not
needed for the pre-built binary install):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Visit [rustup.rs](https://rustup.rs) for platform-specific instructions.

**Xcode 15+ and macOS 14 (Sonoma) or later** — required only if you want the
macOS app.

## CLI

### Install

#### Option 1: Pre-built binary (recommended)

Paste this into Terminal — it downloads the latest release, verifies the
checksum, and installs `drumscribe-index` to `/usr/local/bin`:

```bash
curl -fsSL https://raw.githubusercontent.com/Xapphire13/drumscribe-index/master/scripts/install.sh | bash -s -- --cli-only
```

Use `--cli-dir <path>` to install to a custom directory (e.g.
`--cli-dir ~/.local/bin` if `/usr/local/bin` requires `sudo`). To update, run
the same command again.

#### Option 2: Install with Cargo

```bash
cargo install --git https://github.com/Xapphire13/drumscribe-index.git --path cli
```

Installs `drumscribe-index` to your Cargo bin directory (`~/.cargo/bin/`),
making it available system-wide.

#### Option 3: Build from source

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

### Install

Paste this into Terminal — it downloads the latest release, verifies the
checksum, and installs `Drumscribe Index` to `/Applications`:

```bash
curl -fsSL https://raw.githubusercontent.com/Xapphire13/drumscribe-index/master/scripts/install.sh | bash
```

To also install the `drumscribe-index` CLI binary to `/usr/local/bin`:

```bash
curl -fsSL https://raw.githubusercontent.com/Xapphire13/drumscribe-index/master/scripts/install.sh | bash -s -- --cli
```

Use `--cli-dir <path>` to install the CLI to a custom directory. To update,
run the same command again.

### Build from source

Open in Xcode:

```bash
open ui/DrumscribeIndex.xcodeproj
```

Or build from the command line:

```bash
cd ui && xcodebuild -scheme DrumscribeIndex build
```

### Creating a Release

Before releasing, bump the version with `scripts/bump.sh`:

```bash
# UI-only change (bug fix)
./scripts/bump.sh --ui patch

# UI-only change (new feature)
./scripts/bump.sh --ui minor

# CLI change (always bumps both CLI and UI)
./scripts/bump.sh --cli minor
```

1. **Export the app from Xcode**: `Product > Archive`, then in the Organizer
   select the archive and click **Distribute App > Direct Distribution**.
   Xcode will produce a `Drumscribe Index.app`.

2. **Install `create-dmg`** (one-time):

    ```bash
    brew install create-dmg
    ```

3. **Run the DMG script**:

    ```bash
    ./scripts/make-dmg.sh "/path/to/Drumscribe Index.app"
    ```

   This creates `Drumscribe Index.dmg` in the same directory as the `.app`.

4. **Generate a checksum**:

    ```bash
    shasum -a 256 "Drumscribe Index.dmg" > "Drumscribe Index.dmg.sha256"
    ```

5. **Build the CLI binaries** (run from the repo root; requires both macOS
   cross-compilation targets to be installed):

    ```bash
    ./scripts/make-cli-release.sh --output-dir /path/to/release-dir
    ```

   This produces four files: `drumscribe-index-aarch64-apple-darwin.gz`,
   `drumscribe-index-x86_64-apple-darwin.gz`, and their `.sha256` counterparts.

6. **Create a GitHub release** and upload all 6 assets:
   `Drumscribe Index.dmg`, `Drumscribe Index.dmg.sha256`, and the 4 files from
   the previous step.

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
