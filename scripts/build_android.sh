#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Auto-detect Android Studio Java if JAVA_HOME is not set
if [ -z "$JAVA_HOME" ]; then
    if [ -d "/Applications/Android Studio.app/Contents/jbr/Contents/Home" ]; then
        export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
        export PATH="$JAVA_HOME/bin:$PATH"
    fi
fi

# Auto-detect Android SDK if ANDROID_HOME is not set
if [ -z "$ANDROID_HOME" ]; then
    if [ -d "$HOME/Library/Android/sdk" ]; then
        export ANDROID_HOME="$HOME/Library/Android/sdk"
        export PATH="$ANDROID_HOME/platform-tools:$PATH"
    fi
fi

echo "=== Building atv-android native library ==="

cd "$ROOT_DIR"

BUILD_MODE="${1:-release}"
KEYSTORE_PATH="${2:-$ROOT_DIR/Corvo_Development.p12}"
KEYSTORE_PASS="${3:-${MACOS_CERT_P12_PASSWORD:-corvo-developer}}"

TARGET="aarch64-linux-android"
JNILIBS_DIR="$ROOT_DIR/android-tv/app/src/main/jniLibs/arm64-v8a"
mkdir -p "$JNILIBS_DIR"

if command -v cargo-ndk &> /dev/null; then
    echo "Found cargo-ndk, building arm64-v8a release..."
    cargo ndk -t arm64-v8a -o "$ROOT_DIR/android-tv/app/src/main/jniLibs" build --release -p atv-android
else
    echo "cargo-ndk not found. Checking standard cargo build..."
    cargo build --target "$TARGET" --release -p atv-android
    cp "$ROOT_DIR/target/$TARGET/release/libatv_android.so" "$JNILIBS_DIR/"
fi

echo "Native library copied to: $JNILIBS_DIR/libatv_android.so"

echo "=== Building Android TV APK ($BUILD_MODE) ==="
cd "$ROOT_DIR/android-tv"

if [ "$BUILD_MODE" = "release" ]; then
    ./gradlew assembleRelease
    
    UNSIGNED_APK="$ROOT_DIR/android-tv/app/build/outputs/apk/release/app-release-unsigned.apk"
    SIGNED_APK="$ROOT_DIR/android-tv/app/build/outputs/apk/release/FakeAtv-release.apk"
    
    if [ -f "$KEYSTORE_PATH" ]; then
        echo "🔏 Signing APK with keystore: $KEYSTORE_PATH..."
        # Locate apksigner from Android SDK
        APKSIGNER_BIN=""
        if [ -n "$ANDROID_HOME" ] && [ -d "$ANDROID_HOME/build-tools" ]; then
            APKSIGNER_BIN=$(find "$ANDROID_HOME/build-tools" -name apksigner | sort -V | tail -n1)
        fi
        
        if [ -n "$APKSIGNER_BIN" ] && [ -x "$APKSIGNER_BIN" ]; then
            "$APKSIGNER_BIN" sign \
                --ks "$KEYSTORE_PATH" \
                --ks-type PKCS12 \
                --ks-pass "pass:$KEYSTORE_PASS" \
                --out "$SIGNED_APK" \
                "$UNSIGNED_APK"
            
            echo "🔍 Verifying APK signature..."
            "$APKSIGNER_BIN" verify -v "$SIGNED_APK"
            echo "✅ Signed APK created at: $SIGNED_APK"
        else
            echo "⚠️ apksigner not found in ANDROID_HOME/build-tools. Left APK as unsigned: $UNSIGNED_APK"
        fi
    else
        echo "ℹ️ Keystore $KEYSTORE_PATH not found, keeping unsigned APK at $UNSIGNED_APK"
    fi
else
    ./gradlew assembleDebug
fi

echo "Done!"

