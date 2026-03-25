#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CARGO_TOML="$REPO_ROOT/cli/Cargo.toml"
PBXPROJ="$REPO_ROOT/ui/DrumscribeIndex.xcodeproj/project.pbxproj"

usage() {
  echo "Usage: $0 --cli <patch|minor|major>" >&2
  echo "       $0 --ui <patch|minor|major>" >&2
  echo "       $0 --cli <patch|minor|major> --ui <patch|minor|major>" >&2
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

CLI_BUMP=""
UI_BUMP=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --cli) CLI_BUMP="$2"; shift 2 ;;
    --ui)  UI_BUMP="$2";  shift 2 ;;
    *) usage ;;
  esac
done

if [[ -z "$CLI_BUMP" && -z "$UI_BUMP" ]]; then
  usage
fi

for bump in "${CLI_BUMP:-}" "${UI_BUMP:-}"; do
  if [[ -n "$bump" ]]; then
    case "$bump" in
      patch|minor|major) ;;
      *) echo "Error: bump type must be patch, minor, or major" >&2; exit 1 ;;
    esac
  fi
done

if [[ -n "$CLI_BUMP" ]]; then
  old_cli="$(read_cli_version)"
  new_cli="$(bump_version "$old_cli" "$CLI_BUMP")"
  update_cli_version "$new_cli"
fi

if [[ -n "$UI_BUMP" ]]; then
  old_ui="$(read_ui_version)"
  new_ui="$(bump_version "$old_ui" "$UI_BUMP")"
  update_ui_version "$new_ui"
fi

new_build="$(update_build_number)"

git add "$CARGO_TOML" "$PBXPROJ"

commit_msg="Bump"
[[ -n "$CLI_BUMP" ]] && commit_msg+=" CLI to $new_cli"
[[ -n "$CLI_BUMP" && -n "$UI_BUMP" ]] && commit_msg+=","
[[ -n "$UI_BUMP"  ]] && commit_msg+=" UI to $new_ui"
git commit -m "$commit_msg"

[[ -n "$CLI_BUMP" ]] && git tag -a "cli/v$new_cli" -m "cli v$new_cli"
[[ -n "$UI_BUMP"  ]] && git tag -a "ui/v$new_ui"   -m "ui v$new_ui"

[[ -n "$CLI_BUMP" ]] && echo "CLI: $old_cli → $new_cli"
if [[ -n "$UI_BUMP" ]]; then
  echo "UI:  $old_ui → $new_ui (build $new_build)"
else
  echo "(build $new_build)"
fi
