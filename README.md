# Fake Apple TV (atv-core)

A high-performance Rust implementation of the Apple TV Companion Link / MRP protocol server. It allows real Apple devices (iPhone / iPad Control Center **Apple TV Remote**) to discover, pair, and control:

1. **macOS (Mac mini / MacBook / iMac)** natively via CoreGraphics events, audio controls, and display management.
2. **Android TV** via:
   - **Direct Native App (APK)**: Runs directly on the Android TV using an AccessibilityService + JNI (`atv-android`).
   - **Remote Network ADB**: Runs on macOS/Linux and forwards commands directly to the Android TV over network ADB.

---

## Workspace Structure

```
├── crates/
│   ├── atv-core/        # Pure Rust core: Bonjour (mDNS), MRP, Companion Link, SRP pairing, OPACK, Crypto
│   ├── atv-cli/         # Multi-target CLI executable (macOS native, Android ADB, or print)
│   └── atv-android/     # Rust cdylib providing JNI bindings for Android TV
├── android-tv/          # Android TV Leanback Studio project (APK with AccessibilityService & UI)
└── scripts/
    ├── install_mac.sh   # Build and configure background launchd service for Mac mini
    └── build_android.sh # Cross-compile libatv_android.so and build the Android TV APK
```

---

## 1. macOS (Mac mini) Usage

### Direct Testing (Foreground)
Run the CLI on your Mac:
```bash
# Start server with macOS control and auto-open Debug UI in browser:
cargo run -p atv-cli -- --target mac --name "Mac mini" --pin 1111 --open
```

Options:
- `--open`: Automatically opens the Web Debug UI in your default browser.
- `--ui-port <PORT>`: Custom port for the Web Inspector (default: `8765`).
- `--no-ui`: Disable the Web Inspector server.
- `--mouse`: Enable trackpad mouse mode (finger drag moves pointer, tap clicks).
- `--name "<Name>"`: Change device name shown in iPhone Control Center.
- `--pin <0000-9999>`: Custom pairing PIN code.

> **Important (Accessibility Permission)**:
> Simulated keyboard and mouse events require Accessibility permissions in macOS.
> Go to: **System Settings > Privacy & Security > Accessibility** and ensure your Terminal / iTerm / `atv-cli` is enabled.

### Install as Background Service (Mac mini)
Run the automated installation script:
```bash
./scripts/install_mac.sh
```
This builds the release binary, places it in `~/.local/bin/atv-cli`, and generates a `launchd` plist at `~/Library/LaunchAgents/com.corvofeng.atv-cli.plist`.

- **Start service**: `launchctl load ~/Library/LaunchAgents/com.corvofeng.atv-cli.plist`
- **Stop service**: `launchctl unload ~/Library/LaunchAgents/com.corvofeng.atv-cli.plist`
- **View logs**: `tail -f /tmp/atv-cli.log`

---

## 2. Web Inspector & Debug UI

Whenever `atv-cli` runs (unless `--no-ui` is specified), a built-in real-time Web Inspector is available at:

👉 **`http://127.0.0.1:8765`** (or access from your phone/tablet at `http://<LAN_IP>:8765`)

Features:
- **Interactive Virtual Remote**: Click virtual buttons (D-pad Up/Down/Left/Right, Select, Back, Home, Play/Pause, Mute, Volume +/-) on the webpage to directly test system responses on your Mac or Android TV without needing your phone.
- **Real-Time Touchpad Preview**: Live canvas renders finger swipe paths, displacement (`dx`, `dy`), and touch phases (`Began`, `Moved`, `Ended`).
- **Volume & Audio Gauge**: Live visualization of system volume percentage and mute state.
- **Protocol & Event Log Stream**: Server-Sent Events (SSE) stream raw protocol frames, authentication steps, and connection milestones with millisecond timestamps.

---

## 3. Android TV Usage

### Option A: Direct Android TV App (APK)

Install the app directly on your Android TV box/stick so your TV becomes discoverable as an Apple TV independently (no PC needed).

#### Build & Install:
1. Cross-compile the native Rust library for Android:
   ```bash
   ./scripts/build_android.sh
   ```
2. Or build using Android Studio by opening the `android-tv` directory.
3. Install the APK to your TV:
   ```bash
   adb connect <TV_IP>:5555
   adb install -r android-tv/app/build/outputs/apk/debug/app-debug.apk
   ```
4. Open **Apple TV Remote Receiver** on your Android TV:
   - The screen shows the TV's IP, current status, and the 4-digit pairing PIN (`1111`).
   - Click **Accessibility Settings** on the TV screen and enable the **Apple TV Remote Receiver** accessibility service. This grants permission to simulate Back, Home, Power, and D-pad actions without requiring root!

### Option B: Remote ADB Mode (via Mac mini / PC)

If you prefer to run the server on your Mac mini and forward all inputs to your Android TV over Wi-Fi ADB:

```bash
cargo run -p atv-cli -- --target android --android-host 192.168.1.100
```
- Automatically establishes a persistent `adb shell` connection.
- Extremely low latency (<10ms).
- Trackpad mouse mode (`--mouse`) sends `input roll` and `input tap` to the TV.

---

## Protocol & Key Mapping

| Apple TV Remote Control | macOS (Mac mini) Action | Android TV Action |
| :--- | :--- | :--- |
| **Up / Down / Left / Right** | Arrow Up / Down / Left / Right | `KEYCODE_DPAD_UP` / `DOWN` / `LEFT` / `RIGHT` |
| **Select (Center Click)** | Return (Enter) | `KEYCODE_DPAD_CENTER` / Node Click |
| **Back / Menu (<)** | Escape | `GLOBAL_ACTION_BACK` (`KEYCODE_BACK`) |
| **Home (TV Icon)** | Home key (or configurable) | `GLOBAL_ACTION_HOME` (`KEYCODE_HOME`) |
| **Play / Pause** | Space | `KEYCODE_MEDIA_PLAY_PAUSE` |
| **Volume Up / Down** | Adjust Master Output Volume | `KEYCODE_VOLUME_UP` / `DOWN` |
| **Mute** | Toggle System Mute | `KEYCODE_VOLUME_MUTE` |
| **Power** | Display Sleep (`pmset`) / Wake (`caffeinate`) | `GLOBAL_ACTION_POWER_DIALOG` / `KEYCODE_POWER` |
| **Siri (Side Button)** | Launch Siri (`open -a Siri`) | Voice Search (`KEYCODE_SEARCH`) |
| **Swipe Gestures** | Directional keys or Cursor Move | D-pad navigation or Mouse pointer |
| **Tap (Mouse Mode)** | Left Mouse Click | Touch tap on screen |
