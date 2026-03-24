#!/bin/bash
set -euo pipefail

BINARY_NAME="drumscribe-index"
TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
)

# --- Parse arguments ---

OUTPUT_DIR="$(pwd)"
while [[ $# -gt 0 ]]; do
  case $1 in
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      echo "Usage: $0 [--output-dir <dir>]" >&2
      exit 1
      ;;
  esac
done

mkdir -p "$OUTPUT_DIR"

# --- Resolve CLI directory ---

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CLI_DIR="$SCRIPT_DIR/../cli"

if [[ ! -f "$CLI_DIR/Cargo.toml" ]]; then
  echo "Error: could not find cli/Cargo.toml relative to this script." >&2
  exit 1
fi

# --- Build and package ---

for TARGET in "${TARGETS[@]}"; do
  echo "Building $TARGET..."
  cargo build --release --manifest-path "$CLI_DIR/Cargo.toml" --target "$TARGET"

  SRC="$CLI_DIR/target/$TARGET/release/$BINARY_NAME"
  DEST="$OUTPUT_DIR/${BINARY_NAME}-${TARGET}"

  cp "$SRC" "$DEST"

  echo "Compressing ${BINARY_NAME}-${TARGET}..."
  gzip -f "$DEST"
  # gzip -f replaces $DEST with $DEST.gz

  echo "Generating checksum..."
  shasum -a 256 "${DEST}.gz" > "${DEST}.gz.sha256"

  echo "  ${DEST}.gz"
  echo "  ${DEST}.gz.sha256"
done

echo ""
echo "Done. Upload these 4 files to the GitHub release:"
for TARGET in "${TARGETS[@]}"; do
  echo "  $OUTPUT_DIR/${BINARY_NAME}-${TARGET}.gz"
  echo "  $OUTPUT_DIR/${BINARY_NAME}-${TARGET}.gz.sha256"
done
