# Drumscribe Index

A command-line tool that fetches and generates an index of drum transcription
songs from Drumscribe. It retrieves drum transcription posts and generates a
searchable index with artist, title, difficulty level, and links to full
transcriptions. The tool caches results locally for faster subsequent runs and
supports incremental updates, with output available in JSON, Markdown, HTML, and
XLSX formats.

## Features

- Fetches drum transcription data from the Drumscribe BuyMeACoffee page
- Generates indexes in multiple formats: JSON, Markdown, HTML, and Excel (XLSX)
- Local caching for improved performance
- Incremental updates to keep your index current
- Organizes songs by artist with difficulty ratings
- Supports output to files or standard output

## Installation

### Prerequisites

You'll need to install Rust on your system. Don't worry if you haven't used Rust
before - the installation is straightforward:

1. **Install Rust**: Visit [https://rustup.rs](https://rustup.rs) and follow the
   instructions for your operating system.
   
   - On macOS/Linux, run: ```bash curl --proto '=https' --tlsv1.2 -sSf
     https://sh.rustup.rs | sh ```
   
   - On Windows, download and run the installer from the website.

2. **Verify the installation**: After installation, close and reopen your
   terminal, then run: ```bash cargo --version ``` You should see the Cargo
   version number displayed.

### Installing Drumscribe Index

Once Rust is installed, you have two options:

#### Option 1: Install directly with Cargo (recommended)

```bash
cargo install --git https://github.com/Xapphire13/drumscribe-index.git
```

This installs the program directly from the repository to your Cargo bin directory (usually `~/.cargo/bin/`), making it available system-wide as `drumscribe-index`.

#### Option 2: Build from source

1. **Clone this repository**:
   ```bash
   git clone https://github.com/Xapphire13/drumscribe-index.git
   cd drumscribe-index
   ```

2. **Build the program**:
   ```bash
   cargo build --release
   ```
   This will compile the program. The first build may take a few minutes as it downloads and compiles dependencies.

3. **The compiled program** will be located at:
   ```
   target/release/drumscribe-index
   ```

## Usage

### Basic Usage

Run the program to generate a JSON index (printed to your terminal):

```bash 
cargo run --release
```

Or use the compiled binary directly:

```bash
./target/release/drumscribe-index
```

### Output Formats

Choose your preferred output format using command-line flags:

**JSON (default)**:
```bash
drumscribe-index --json 
```

**Markdown**:
```bash
drumscribe-index --markdown 
```

**HTML**:
```bash
drumscribe-index --html
```

**Excel (XLSX)**: Requires specifying an output file
```bash
drumscribe-index --xlsx --output songs.xlsx
```

### Saving to a File

Use the `--output` flag to save results to a file:

```bash
drumscribe-index --markdown --output index.md
drumscribe-index --html --output index.html
drumscribe-index --json --output songs.json
```

### Updating the Index

The first time you run the program, it will fetch all available songs and cache
them locally. On subsequent runs, it uses the cached data for faster
performance.

To update your index with new songs published since your last run:

```bash
drumscribe-index --update
```

You can combine `--update` with any output format:

```bash
drumscribe-index --update --xlsx --output songs.xlsx
```

### Examples

1. **Create a Markdown file with all songs**:
```bash
drumscribe-index --markdown --output drumscribe-songs.md
 ```

2. **Update your index and export to Excel**:
```bash
drumscribe-index --update --xlsx --output songs.xlsx
 ```

3. **Generate an HTML page**:
```bash
drumscribe-index --html --output index.html
```

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

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
