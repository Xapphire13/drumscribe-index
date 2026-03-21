#!/bin/bash
set -e

if [ -z "$1" ]; then
  echo "Usage: $0 <path/to/DrumscribeIndex.app>"
  exit 1
fi

APP_PATH="$1"
APP_NAME=$(basename "$APP_PATH" .app)
DMG_PATH="$(dirname "$APP_PATH")/${APP_NAME}.dmg"

create-dmg \
  --volname "$APP_NAME" \
  --window-pos 200 120 \
  --window-size 600 400 \
  --icon-size 100 \
  --icon "${APP_NAME}.app" 150 185 \
  --app-drop-link 450 185 \
  "$DMG_PATH" \
  "$APP_PATH"

echo "Created: $DMG_PATH"
