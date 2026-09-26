#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=========================================="
echo "💿 Building Apple TV Remote DMG Installer"
echo "=========================================="

# 1. Build .app bundle first
"$SCRIPT_DIR/build_app.sh"

BUILD_DIR="$ROOT_DIR/build"
APP_BUNDLE="$BUILD_DIR/AppleTVRemote.app"
STAGING_DIR="$BUILD_DIR/dmg_staging"
DMG_PATH="$BUILD_DIR/AppleTVRemote.dmg"

if [ ! -d "$APP_BUNDLE" ]; then
    echo "❌ Error: $APP_BUNDLE does not exist."
    exit 1
fi

echo "📁 Staging DMG contents..."
rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR"

# Copy App to staging
cp -R "$APP_BUNDLE" "$STAGING_DIR/"

# Create symlink to /Applications for easy drag-and-drop install
ln -s /Applications "$STAGING_DIR/Applications"

# Remove existing DMG if present
rm -f "$DMG_PATH"

echo "💿 Creating DMG image with hdiutil..."
hdiutil create \
    -volname "Apple TV Remote" \
    -srcfolder "$STAGING_DIR" \
    -ov \
    -format UDZO \
    "$DMG_PATH"

# Clean staging directory
rm -rf "$STAGING_DIR"

echo "=========================================="
echo "🎉 DMG Packaging Complete!"
echo "📦 Output file: $DMG_PATH"
echo "📏 File size: $(ls -lh "$DMG_PATH" | awk '{print $5}')"
echo "=========================================="
