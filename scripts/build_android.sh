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

echo "=== Building Android TV APK ==="
cd "$ROOT_DIR/android-tv"
if [ -f "./gradlew" ]; then
    ./gradlew assembleDebug
elif command -v gradle &> /dev/null; then
    gradle assembleDebug
else
    echo "Gradle not found directly in path. You can open 'android-tv' in Android Studio to build the APK."
fi

echo "Done!"
