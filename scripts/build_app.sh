#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=========================================="
echo "🔨 Building Apple TV Remote for macOS (.app)"
echo "=========================================="

cd "$ROOT_DIR"

# 1. Compile Rust core binary
echo "🦀 [1/4] Compiling Rust core engine (atv-cli)..."
cargo build --release -p atv-cli

# 2. Generate App Icon if missing
if [ ! -f "$ROOT_DIR/mac-app/resources/AppIcon.icns" ]; then
    echo "🎨 [2/4] Generating AppIcon.icns..."
    swift "$ROOT_DIR/scripts/generate_icon.swift"
else
    echo "🎨 [2/4] AppIcon.icns already exists."
fi

# 3. Compile Swift Native App
echo "🍏 [3/4] Compiling Swift Native App..."
BUILD_DIR="$ROOT_DIR/build"
mkdir -p "$BUILD_DIR"

SWIFT_BIN="$BUILD_DIR/AppleTVRemote"
swiftc -O -target arm64-apple-macos12.0 \
    "$ROOT_DIR"/mac-app/src/*.swift \
    -o "$SWIFT_BIN"

# 4. Assemble macOS .app Bundle
APP_BUNDLE="$BUILD_DIR/AppleTVRemote.app"
echo "📦 [4/4] Assembling $APP_BUNDLE..."
rm -rf "$APP_BUNDLE"

mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Copy binaries
cp "$SWIFT_BIN" "$APP_BUNDLE/Contents/MacOS/AppleTVRemote"
cp "$ROOT_DIR/target/release/atv-cli" "$APP_BUNDLE/Contents/MacOS/atv-cli"
chmod +x "$APP_BUNDLE/Contents/MacOS/AppleTVRemote"
chmod +x "$APP_BUNDLE/Contents/MacOS/atv-cli"

# Copy metadata and resources
cp "$ROOT_DIR/mac-app/Info.plist" "$APP_BUNDLE/Contents/Info.plist"
echo "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"
cp "$ROOT_DIR/mac-app/resources/AppIcon.icns" "$APP_BUNDLE/Contents/Resources/AppIcon.icns"

# Clean temporary standalone swift binary
rm -f "$SWIFT_BIN"

# 5. Code sign application bundle (Auto-detecting best certificate)
find_best_identity() {
    # 1. 优先使用环境变量指定的证书
    if [ -n "$CODESIGN_IDENTITY" ]; then
        if security find-identity -v -p codesigning | grep -q "\"$CODESIGN_IDENTITY\""; then
            echo "$CODESIGN_IDENTITY"
            return
        fi
    fi

    # 2. 匹配 Apple 官方发布证书 (Developer ID Application)
    local dev_id
    dev_id=$(security find-identity -v -p codesigning | grep "Developer ID Application:" | head -n1 | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -n "$dev_id" ]; then
        echo "$dev_id"
        return
    fi

    # 3. 匹配 Apple 官方开发证书 (Apple Development)
    local apple_dev
    apple_dev=$(security find-identity -v -p codesigning | grep "Apple Development:" | head -n1 | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -n "$apple_dev" ]; then
        echo "$apple_dev"
        return
    fi

    # 4. 匹配本地通用持久化自签证书 (Corvo Development)
    if security find-identity -v -p codesigning | grep -q "\"Corvo Development\""; then
        echo "Corvo Development"
        return
    fi

    # 5. 兼容历史项目证书 (AppleTVRemote Development)
    if security find-identity -v -p codesigning | grep -q "\"AppleTVRemote Development\""; then
        echo "AppleTVRemote Development"
        return
    fi

    echo ""
}

SIGN_IDENTITY=$(find_best_identity)

if [ -n "$SIGN_IDENTITY" ]; then
    echo "🔏 Signing application with certificate: \"$SIGN_IDENTITY\"..."
    codesign --force --sign "$SIGN_IDENTITY" --identifier "com.corvofeng.AppleTVRemote.helper" "$APP_BUNDLE/Contents/MacOS/atv-cli"
    codesign --force --sign "$SIGN_IDENTITY" --identifier "com.corvofeng.AppleTVRemote" "$APP_BUNDLE/Contents/MacOS/AppleTVRemote"
    codesign --force --sign "$SIGN_IDENTITY" "$APP_BUNDLE"
    echo "✨ Signed successfully with \"$SIGN_IDENTITY\"! Permissions will survive rebuilds."
else
    echo "🔏 Signing application bundle (ad-hoc fallback)..."
    echo "💡 提示: 可运行 ./scripts/setup_certificate.sh 创建本地持久化证书，重编后权限永不失效。"
    codesign --force --sign - --identifier "com.corvofeng.AppleTVRemote.helper" "$APP_BUNDLE/Contents/MacOS/atv-cli"
    codesign --force --sign - --identifier "com.corvofeng.AppleTVRemote" "$APP_BUNDLE/Contents/MacOS/AppleTVRemote"
    codesign --force --sign - "$APP_BUNDLE"
fi

echo "=========================================="
echo "✅ Successfully built: $APP_BUNDLE"
echo "=========================================="
