#!/bin/bash
set -euo pipefail

REPO="Xapphire13/drumscribe-index"
APP_NAME="Drumscribe Index"
CLI_BINARY_NAME="drumscribe-index"
APP_INSTALL_DIR="/Applications"

# --- Parse arguments ---

WAIT_PID=""
INSTALL_CLI=false
CLI_ONLY=false
CLI_DIR="/usr/local/bin"

while [[ $# -gt 0 ]]; do
  case $1 in
    --wait-pid)
      WAIT_PID="$2"
      shift 2
      ;;
    --cli)
      INSTALL_CLI=true
      shift
      ;;
    --cli-only)
      INSTALL_CLI=true
      CLI_ONLY=true
      shift
      ;;
    --cli-dir)
      CLI_DIR="$2"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      echo "Usage: $0 [--wait-pid <pid>] [--cli] [--cli-only] [--cli-dir <dir>]" >&2
      exit 1
      ;;
  esac
done

# --- Wait for caller process to exit (update flow) ---

if [[ -n "$WAIT_PID" ]]; then
  echo "Waiting for process $WAIT_PID to exit..."
  while kill -0 "$WAIT_PID" 2>/dev/null; do
    sleep 0.5
  done
fi

# --- Helpers ---

download_verify() {
  local url="$1" checksum_url="$2" dest="$3"
  curl -fL --progress-bar --output "$dest" "$url"
  local checksum_file="$dest.sha256"
  curl -fsSL --output "$checksum_file" "$checksum_url"
  local expected actual
  expected=$(awk '{print $1}' "$checksum_file")
  actual=$(shasum -a 256 "$dest" | awk '{print $1}')
  if [[ "$expected" != "$actual" ]]; then
    echo "Error: checksum mismatch for $(basename "$dest")!" >&2
    echo "  Expected: $expected" >&2
    echo "  Actual:   $actual" >&2
    exit 1
  fi
  echo "Checksum OK."
}

# --- Fetch latest release metadata ---

echo "Fetching latest release info..."
RELEASE_JSON=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")

# --- Set up temp directory and cleanup ---

TMP_DIR=$(mktemp -d)
cleanup() {
  hdiutil detach "$TMP_DIR/mount" -quiet 2>/dev/null || true
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# --- Install app (unless --cli-only) ---

if [[ "$CLI_ONLY" != true ]]; then
  DMG_URL=$(echo "$RELEASE_JSON" \
    | grep -o '"browser_download_url": *"[^"]*\.dmg"' \
    | grep -o 'https://[^"]*')

  CHECKSUM_URL=$(echo "$RELEASE_JSON" \
    | grep -o '"browser_download_url": *"[^"]*\.dmg\.sha256"' \
    | grep -o 'https://[^"]*')

  if [[ -z "$DMG_URL" ]]; then
    echo "Error: no .dmg asset found in the latest release." >&2
    exit 1
  fi
  if [[ -z "$CHECKSUM_URL" ]]; then
    echo "Error: no .dmg.sha256 asset found in the latest release." >&2
    exit 1
  fi

  DMG_FILE="$TMP_DIR/${APP_NAME}.dmg"

  echo "Downloading ${APP_NAME}.dmg..."
  download_verify "$DMG_URL" "$CHECKSUM_URL" "$DMG_FILE"

  echo "Mounting disk image..."
  mkdir "$TMP_DIR/mount"
  hdiutil attach -nobrowse -quiet -mountpoint "$TMP_DIR/mount" "$DMG_FILE"

  echo "Installing ${APP_NAME}.app to ${APP_INSTALL_DIR}..."
  if [[ -d "${APP_INSTALL_DIR}/${APP_NAME}.app" ]]; then
    rm -rf "${APP_INSTALL_DIR}/${APP_NAME}.app"
  fi
  cp -R "$TMP_DIR/mount/${APP_NAME}.app" "${APP_INSTALL_DIR}/"

  echo "${APP_NAME} installed successfully."
fi

# --- Install CLI binary (if --cli or --cli-only) ---

if [[ "$INSTALL_CLI" == true ]]; then
  ARCH=$(uname -m)
  case $ARCH in
    arm64)  CLI_ARCH="aarch64-apple-darwin" ;;
    x86_64) CLI_ARCH="x86_64-apple-darwin" ;;
    *)
      echo "Error: unsupported architecture: $ARCH" >&2
      exit 1
      ;;
  esac

  CLI_ASSET="${CLI_BINARY_NAME}-${CLI_ARCH}.gz"

  CLI_GZ_URL=$(echo "$RELEASE_JSON" \
    | grep -o "\"browser_download_url\": *\"[^\"]*${CLI_ARCH}\\.gz\"" \
    | grep -o 'https://[^"]*')

  CLI_CHECKSUM_URL=$(echo "$RELEASE_JSON" \
    | grep -o "\"browser_download_url\": *\"[^\"]*${CLI_ARCH}\\.gz\\.sha256\"" \
    | grep -o 'https://[^"]*')

  if [[ -z "$CLI_GZ_URL" ]]; then
    echo "Error: no CLI binary asset found for ${CLI_ARCH} in the latest release." >&2
    exit 1
  fi
  if [[ -z "$CLI_CHECKSUM_URL" ]]; then
    echo "Error: no CLI checksum asset found for ${CLI_ARCH} in the latest release." >&2
    exit 1
  fi

  CLI_GZ_FILE="$TMP_DIR/$CLI_ASSET"

  echo "Downloading ${CLI_ASSET}..."
  download_verify "$CLI_GZ_URL" "$CLI_CHECKSUM_URL" "$CLI_GZ_FILE"

  echo "Decompressing..."
  gunzip "$CLI_GZ_FILE"
  CLI_BIN_FILE="$TMP_DIR/${CLI_BINARY_NAME}-${CLI_ARCH}"
  chmod +x "$CLI_BIN_FILE"

  if [[ ! -d "$CLI_DIR" ]]; then
    echo "Error: CLI install directory does not exist: $CLI_DIR" >&2
    exit 1
  fi
  if [[ ! -w "$CLI_DIR" ]]; then
    echo "Error: $CLI_DIR is not writable." >&2
    echo "  Try: sudo $0 $* " >&2
    echo "  Or:  $0 --cli-dir ~/.local/bin" >&2
    exit 1
  fi

  cp "$CLI_BIN_FILE" "$CLI_DIR/$CLI_BINARY_NAME"
  echo "CLI installed to $CLI_DIR/$CLI_BINARY_NAME"
fi
