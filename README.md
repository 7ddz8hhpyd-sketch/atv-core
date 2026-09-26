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
├── mac-app/             # Native macOS Menu Bar Application (Swift AppKit)
│   ├── Info.plist       # macOS App Bundle metadata
│   ├── resources/       # Multi-resolution Retina AppIcon.icns
│   └── src/             # Swift sources (StatusItem, ProcessManager, ApiClient, Accessibility)
├── android-tv/          # Android TV Leanback Studio project (APK with AccessibilityService & UI)
└── scripts/
    ├── build_app.sh     # One-click build for standalone AppleTVRemote.app bundle
    ├── build_dmg.sh     # One-click packaging for AppleTVRemote.dmg installer
    ├── generate_icon.swift # Generates high-res macOS Retina AppIcon.icns
    ├── install_mac.sh   # Build and configure background launchd service for Mac mini
    └── build_android.sh # Cross-compile libatv_android.so and build the Android TV APK
```

---

## 0. macOS Native App & DMG Packaging (Swift + Rust)

本项目采用了**内核与外层彻底分离**的现代化架构：
* **核心协议内核 (Rust)**：仅负责高性能网络通信、Bonjour 广播、SRP 安全配对、Companion Link 加密链路与手势事件分发。
* **原生系统层封装 (Swift)**：采用原生 Swift + AppKit 打造轻量级 macOS 菜单栏状态应用（Menu Bar Agent App），负责应用生命周期、菜单栏实时状态、模式/速度切换、辅助功能权限提示与系统通知。

### 📦 一键构建原生 `.app` 与 `.dmg` 安装包

```bash
# 1. 编译并打包为 macOS 应用程序 (build/AppleTVRemote.app)
./scripts/build_app.sh

# 2. 编译并打包为磁盘映像安装包 (build/AppleTVRemote.dmg)
./scripts/build_dmg.sh
```

生成的 `AppleTVRemote.dmg` 自带 `/Applications` 软链接，直接拖拽即可完成安装：
* 启动后常驻 macOS 菜单栏，带原生 Apple TV 遥控器图标与实时状态徽标。
* 点击菜单栏图标可随时切换「🖱️ 鼠标光标模式」与「◀▲▼▶ 方向键模式」，并可一键调节 0.5x ~ 2.0x 灵敏度预设与动态加速度。
* 当 iPhone 连接或断开时，系统会自动弹出原生通知横幅提醒。

---

## 1. macOS Usage

### Direct Testing (Foreground)
Run the CLI on your Mac:
```bash
# Starts server with macOS control (auto-detects your Mac's ComputerName):
cargo run -p atv-cli -- --target mac --pin 1111 --open
```

Features on macOS:
- **自动读取 Mac 机器名 (Auto Device Name)**: 自动通过 `scutil --get ComputerName` 获取当前 Mac 电脑名称并发布 Bonjour 广播（如 `Yuhao's MacBook`），iPhone 控制中心下拉遥控器直接显示本机名称（无需手动传参，也可用 `--name` 自定义覆盖）。
- **设备专属唯一硬件标识 (Per-Device Unique Identity)**:
  - 自动读取 macOS 硬件 UUID (`IOPlatformUUID`) 作为 Apple TV 专属配对识别码（`server_identifier` 与 AirPlay `pi`）。
  - 自动读取本机真实网卡 MAC 地址（Wi-Fi / Ethernet）作为硬件设备 ID（`device_id` / `rpBA` / `rpHA`）。
  - 基于设备硬件唯一标识生成持久私钥种子与 Bonjour 服务校验参数，**多台 Mac / 设备在同一局域网下各自拥有独立 ID，不再共享相同 ID**，彻底避免设备识别冲突或 iPhone 配对记录相互覆盖，且重启后配对凭证持续生效。
- **程序内动态模式切换 (Directional Keys vs Mouse Cursor)**:
  - **方向键模式 (上下左右)**: 遥控器滑动转换为方向键（↑ ↓ ← →），轻触中心或按下 **SELECT** 键为回车确认（Return / Enter）。
  - **超流畅鼠标光标模式 (Smooth Mouse Cursor)**:
    - 采用**本地坐标累加器 (Local Cursor Accumulator)**，彻底消除旧版本高频 IPC 查询 WindowServer 的异步竞态与抖动迟滞。
    - 内置**动态加速度曲线 (Ballistics)**：微小移动（<1.5 像素）提供 1.15x 像素级精准定位，高速甩动自动加速至 4.5x，轻松横跨双屏 / 4K 大屏。
    - 轻触或按下 **SELECT** 键直接执行鼠标左键单击（解决滑动后手抖点歪的问题），连续点击自动触发双击。
  - **切换方式 (三端实时同步)**:
    1. **终端交互**: 在运行中的终端中直接输入 `m` 并回车，随时切换模式。
    2. **Web Inspector**: 在网页控制面板点击切换按钮。
    3. **iPhone 遥控器**: 直接点击遥控器上的 **Siri / 语音键** 快捷切换，屏幕上会同步弹出 macOS 原生通知横幅！
- **系统级多媒体控制 (Multimedia & Volume Controls)**:
  - **播放 / 暂停 (Play / Pause)**: 映射 macOS 原生系统媒体键（`NX_KEYTYPE_PLAY`），联动系统级 Now Playing 守护进程，可直接控制正在播放的 Apple Music、Spotify、Safari / Chrome 网页视频（Bilibili、YouTube、Netflix）、IINA、QuickTime 等。
  - **切歌手势 (Next / Prev)**:
    - 因 iOS 控制中心 Apple TV 遥控器原生未提供单独的「上一曲/下一曲」物理按键，系统参考 Apple EarPods / AirPods 经典手势进行了深度映射：
      - **单击 ⏯ 键**: 播放 / 暂停
      - **双击 ⏯ 键**: 切换下一曲（Next Track ⏭，并在 Mac 上弹出通知）
      - **三击 ⏯ 键**: 切换上一曲（Previous Track ⏮，并在 Mac 上弹出通知）
    - 亦可在 Web Inspector 控制面板中直接点击专属的 `[⏮ Prev]` 与 `[⏭ Next]` 按钮进行调试测试。
  - **系统音量调节 (Volume Up / Down / Mute)**: 映射 macOS 硬件音量键（`NX_KEYTYPE_SOUND_UP` / `DOWN` / `MUTE`），调节时屏幕弹出系统原生音量半透明 HUD 面板，并实时双向同步音量百分比与静音状态到 iPhone 遥控器。

Options:
- `--open`: Automatically opens the Web Debug UI in your default browser.
- `--ui-port <PORT>`: Custom port for the Web Inspector (default: `8765`).
- `--no-ui`: Disable the Web Inspector server.
- `--mouse`: Initial start in trackpad mouse mode (can be toggled at runtime with `m` or from remote/web).
- `--speed <0.2-3.0>`: Initial mouse speed / sensitivity multiplier (default: `0.5`, persisted in `~/.config/atv/settings.json`).
- `--verbose-events`: Enable verbose terminal logging for buttons and events (default: quiet debug).
- `--name "<Name>"`: Custom device name (defaults to current Mac's ComputerName on macOS).
- `--device-id <MAC>`: Custom device MAC identifier (defaults to auto-detected hardware MAC).
- `--server-id <UUID>`: Custom server UUID (defaults to auto-detected hardware UUID).
- `--pin <0000-9999>`: Custom pairing PIN code (default: `1111`).

> **Important (Accessibility Permission)**:
> Simulated keyboard and mouse events require Accessibility permissions in macOS.
> Go to: **System Settings > Privacy & Security > Accessibility** and ensure your Terminal / iTerm / `atv-cli` is enabled.

### Install as Background Service (macOS)
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
- **🎛️ 触摸板独立设置与状态记录 (Touchpad Settings & Record)**:
  - **运行模式一键切换**: 「方向键模式 (上下左右)」与「鼠标光标模式 (Mouse Pointer)」。
  - **动态灵敏度滑块 (0.2x ~ 3.0x)**: 支持像专业鼠标驱动一样无级拖动滑块，即时生效调节光标移动速度。
  - **快速档位预设**: 提供 `0.5x 慢速精准`、`1.0x 标准默认`、`1.5x 快速顺畅`、`2.2x 极速大屏` 4 个常用档位一键直达。
  - **动力学加速度曲线开关 (Ballistics vs Linear)**: 随时一键切换「动态加速度 (微移稳准、甩动极速)」或「1:1 纯线性匀速」。
  - **详细事件日志级别动态控制**:
    - 默认按键与高频事件降级为 `debug` 级别，终端保持干净清爽、不被频繁刷屏；
    - 可在网页上随时一键切换 `🔇 静音 (Debug)` / `📢 详细输出 (Info)`，终端支持按 `v` 键热切换。
  - **📋 实时配置与运行状态记录看板**: 记录当前运行模式、速度倍率、动力学曲线、SELECT 映射、macOS 主屏幕物理分辨率（如 `3840 × 2160`）与最后同步时间。
- **Interactive Virtual Remote**: Click virtual buttons (D-pad Up/Down/Left/Right, Select, Back, Home, Play/Pause, Next/Prev, Mute, Volume +/-) on the webpage to directly test system responses on your Mac or Android TV without needing your phone.
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
