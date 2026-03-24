#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CARGO_TOML="$REPO_ROOT/cli/Cargo.toml"
PBXPROJ="$REPO_ROOT/ui/DrumscribeIndex.xcodeproj/project.pbxproj"

usage() {
  echo "Usage: $0 --cli <patch|minor|major>" >&2
  echo "       $0 --ui <patch|minor|major>" >&2
  exit 1
}

bump_version() {
  local version="$1"
  local bump="$2"
  local major minor patch

  IFS='.' read -r major minor patch <<< "$version"

  case "$bump" in
    major) major=$((major + 1)); minor=0; patch=0 ;;
    minor) minor=$((minor + 1)); patch=0 ;;
    patch) patch=$((patch + 1)) ;;
    *) echo "Error: bump type must be patch, minor, or major" >&2; exit 1 ;;
  esac

  echo "$major.$minor.$patch"
}

read_cli_version() {
  grep '^version = ' "$CARGO_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/'
}

read_ui_version() {
  grep 'MARKETING_VERSION = ' "$PBXPROJ" | head -1 | sed 's/.*MARKETING_VERSION = \(.*\);/\1/'
}

read_build_number() {
  grep 'CURRENT_PROJECT_VERSION = ' "$PBXPROJ" | head -1 | sed 's/.*CURRENT_PROJECT_VERSION = \(.*\);/\1/'
}

update_cli_version() {
  local new_version="$1"
  sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
}

update_ui_version() {
  local new_version="$1"
  sed -i '' "s/MARKETING_VERSION = .*;/MARKETING_VERSION = $new_version;/g" "$PBXPROJ"
}

update_build_number() {
  local current_build new_build
  current_build="$(read_build_number)"
  new_build=$((current_build + 1))
  sed -i '' "s/CURRENT_PROJECT_VERSION = $current_build;/CURRENT_PROJECT_VERSION = $new_build;/g" "$PBXPROJ"
  echo "$new_build"
}

if [[ $# -ne 2 ]]; then
  usage
fi

MODE=""
BUMP=""

case "$1" in
  --cli) MODE="cli"; BUMP="$2" ;;
  --ui)  MODE="ui";  BUMP="$2" ;;
  *) usage ;;
esac

case "$BUMP" in
  patch|minor|major) ;;
  *) echo "Error: bump type must be patch, minor, or major" >&2; exit 1 ;;
esac

if [[ "$MODE" == "cli" ]]; then
  old_cli="$(read_cli_version)"
  new_cli="$(bump_version "$old_cli" "$BUMP")"
  new_ui="$(bump_version "$old_cli" "$BUMP")"  # UI tracks CLI version on CLI bumps

  update_cli_version "$new_cli"
  update_ui_version "$new_ui"
  new_build="$(update_build_number)"

  echo "CLI: $old_cli → $new_cli"
  echo "UI:  $old_cli → $new_ui (build $new_build)"
else
  old_ui="$(read_ui_version)"
  new_ui="$(bump_version "$old_ui" "$BUMP")"

  update_ui_version "$new_ui"
  new_build="$(update_build_number)"

  echo "UI: $old_ui → $new_ui (build $new_build)"
fi
