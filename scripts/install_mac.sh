#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=== Building atv-cli for macOS ==="
cd "$ROOT_DIR"
cargo build --release -p atv-cli

BIN_PATH="$ROOT_DIR/target/release/atv-cli"
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp "$BIN_PATH" "$INSTALL_DIR/atv-cli"
echo "Installed binary to $INSTALL_DIR/atv-cli"

PLIST_PATH="$HOME/Library/LaunchAgents/com.corvofeng.atv-cli.plist"
mkdir -p "$HOME/Library/LaunchAgents"

cat <<EOF > "$PLIST_PATH"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.corvofeng.atv-cli</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/atv-cli</string>
        <string>--target</string>
        <string>mac</string>
        <string>--name</string>
        <string>Mac mini</string>
        <string>--pin</string>
        <string>1111</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/atv-cli.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/atv-cli.err</string>
</dict>
</plist>
EOF

echo "Generated launchd agent: $PLIST_PATH"
echo ""
echo "To start the background service immediately on Mac mini:"
echo "  launchctl load $PLIST_PATH"
echo ""
echo "To stop the background service:"
echo "  launchctl unload $PLIST_PATH"
echo ""
echo "Or run manually in foreground for testing:"
echo "  $INSTALL_DIR/atv-cli --target mac"
