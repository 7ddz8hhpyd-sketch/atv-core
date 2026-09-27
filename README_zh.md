# Fake Apple TV (atv-core) 📺

> [English Version](README.md) | **中文版本**

高性能 Rust 实现的 Apple TV Companion Link / MRP 协议服务端。无需越狱或特殊硬件，即可将真实 Apple 设备（iPhone / iPad 控制中心 **Apple TV 遥控器**）直接发现、配对并遥控：

1. **Android TV / Google TV / 电视盒子 / 模拟器**:
   - **原生 APK 独立运行**: 基于 JNI + 本地 ADB (`dadb`) + 无障碍辅助服务 (`AtvAccessibilityService`) 驱动，电视开机自启，独立对外广播。
   - **远程网络 ADB 模式**: 由 Mac mini / PC 运行服务端，通过网络 ADB 实时向电视转发按键与手势。
2. **macOS (Mac mini / MacBook / iMac)**:
   - 原生 Swift 菜单栏 App (`AppleTVRemote.app`) 与跨平台 CLI。
   - 支持**超流畅自适应动态加速度鼠标光标**与**方向键**双模式无缝切换，原生联动系统多媒体键与硬件音量。
3. **内置 Web 交互控制台 (Web Inspector)**:
   - 浏览器打开即用的 macOS 调试控制面板，实时可视化触摸板轨迹、音量仪表盘、虚拟遥控器测试面板与毫秒级 SSE 协议事件流。

---

## 目录

- [一、核心工作原理与手势流转图](#一核心工作原理与手势流转图)
- [二、前置条件与核心权限核对指南（必读）](#二前置条件与核心权限核对指南必读)
  - [1. Android TV 关键前置要求](#1-android-tv-关键前置要求)
  - [2. macOS 关键前置要求与签名授权](#2-macos-关键前置要求与签名授权)
  - [3. 局域网配对网络要求](#3-局域网配对网络要求)
- [三、Android TV 页面移动与实操体验](#三android-tv-页面移动与实操体验)
  - [1. 遥控器手势与页面焦点移动](#1-遥控器手势与页面焦点移动)
  - [2. Android TV App 控制看板与功能配置](#2-android-tv-app-控制看板与功能配置)
- [四、Android 模拟器联调与 Bridge 桥接操作指南](#四android-模拟器联调与-bridge-桥接操作指南)
  - [1. 模拟器网络通信架构](#1-模拟器网络通信架构)
  - [2. 模拟器操作与桥接步骤](#2-模拟器操作与桥接步骤)
- [五、macOS 原生菜单栏应用与使用指南](#五macos-原生菜单栏应用与使用指南)
- [六、macOS 浏览器实时控制面板 (Web Inspector)](#六macos-浏览器实时控制面板-web-inspector)
- [七、按键与手势映射速查表](#七按键与手势映射速查表)
- [八、常见问题排查 (FAQ)](#八常见问题排查-faq)

---

## 一、核心工作原理与手势流转图

当你在 iPhone 控制中心下拉打开 Apple TV 遥控器轻扫滑屏时，系统通过 TLS 加密的 Companion Link 协议发送高频触控微积分坐标。`atv-core` 解析协议后，在 Android TV 上转换为网格焦点平滑移动，在 macOS 上则转换为带人体工学加速度曲线的光标平移。

![手势与页面移动流程图](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/page_navigation_movement.svg)

---

## 二、前置条件与核心权限核对指南（必读）

为了保障按键与触控能顺利注入系统，请在首次运行前核对以下系统设置与权限：

![前置条件与权限指南](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/prerequisites_and_permissions.svg)

### 1. Android TV 关键前置要求

#### ① 必须开启「网络 ADB 调试」与「USB 调试」
* **为什么必须开启 ADB？**
  * Android 电视为了安全，普通第三方应用无法在全局任意界面（包括桌面、其他影视 App）模拟物理方向键与点击。
  * 本应用内置专有的 Dadb 高性能驱动，会在后台通过本地回环地址直连 `127.0.0.1:5555`，直接向系统底层直接派发事件，达到 **<8ms 极低延迟**的丝滑操控。若关闭 ADB，将无法派发底层物理按键！
* **开启步骤**：
  1. 打开电视 **系统设置 -> 关于 (About)**。
  2. 连续点击 **内部版本号 (Build Number)** 7 次，直到屏幕提示「您现在处于开发者模式」。
  3. 返回上一级菜单，进入 **系统 -> 开发者选项 (Developer Options)**。
  4. 开启 **USB 调试 (USB Debugging)** 以及 **网络 ADB 调试 (Network Debugging)**。

#### ② 必须接受系统 ADB 指纹授权与签名
首次启动应用连接本地 ADB 时，Android TV 屏幕上会弹出系统级安全对话框：

![ADB 授权弹窗示意](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/prerequisites_and_permissions.svg)

> ⚠️ **极为关键**：
> 请务必使用电视遥控器**勾选「始终允许来自此计算机 (Always allow from this computer)」**，然后点击 **「允许 / OK」**。
> 如果误点了取消或拒绝，本地驱动将提示 `Unauthorized`，遥控将无法生效。

#### ③ 开启无障碍服务权限 (Accessibility Service)
* **作用**：提供全局系统返回键 (`GLOBAL_ACTION_BACK`)、主页键 (`GLOBAL_ACTION_HOME`) 与电源弹窗，并在未开启 ADB 时作为备用兜底通道。
* **开启步骤**：
  1. 在电视应用主页点击 **「ACCESSIBILITY SETTINGS」** 按钮。
  2. 在系统列表中找到 **Apple TV Remote Receiver**（默认显示 Off）。
  3. 点击进入，选择开启，在系统弹出的确认窗口中点击 **OK / 确定** 允许权限。

| 步骤 1: 辅助功能列表 | 步骤 2: 授权确认弹窗 | 步骤 3: 授权生效状态 |
| :---: | :---: | :---: |
| ![服务列表](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/android_tv_accessibility_service_list.png) | ![授权确认](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/android_tv_permission_dialog.png) | ![就绪状态](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/android_tv_main_screen_ready.png) |

---

### 2. macOS 关键前置要求与签名授权

#### ① 解决 Gatekeeper 拦截（接受自签名证书）
本项目 Release 预编译包使用开发者专属自签名证书 (`Corvo Development`) 签名。在非构建 Mac 上首次运行时，macOS Gatekeeper 会提示「无法验证此 App 不包含恶意软件」。

请选择以下任意一种方式放行：

* **方式 A（推荐，一劳永逸）**：导入项目内置自签名证书至系统钥匙串：
  ```bash
  ./scripts/import_certificate.sh Corvo_Development.p12
  ```
* **方式 B（快捷单机跳过）**：移除应用的隔离扩展属性：
  ```bash
  xattr -dr com.apple.quarantine /Applications/AppleTVRemote.app
  ```
  *或者：在 Finder 中找到该 App，按住 `Control` 键点击图标，选择「打开」，在二次确认框中点击「打开」。*

#### ② 授予「辅助功能」权限
模拟全局键盘敲击与平滑鼠标移动需要 macOS 系统的辅助功能权限：
1. 打开 **系统设置 -> 隐私与安全性 -> 辅助功能**。
2. 确保勾选允许 **AppleTVRemote**（若使用命令行调试，则勾选终端应用如 **Terminal** 或 **iTerm**）。

---

### 3. 局域网配对网络要求

1. **同一 Wi-Fi 子网**：iPhone / iPad 与 Android TV / Mac 必须接入同一个局域网路由器（2.4G / 5G 频段可互通）。
2. **禁止 AP 隔离**：部分商用路由器开启了「AP 隔离 (Client Isolation)」，会导致设备间无法收发 mDNS (Bonjour UDP 5353) 广播包。请确保路由器允许局域网组播与设备间互访。
3. **初次配对 PIN 码**：iPhone 控制中心点击设备后，电视或终端会展示 4 位验证码，在手机上输入 **`1111`** 即可完成 SRP 握手。

---

## 三、Android TV 页面移动与实操体验

### 1. 遥控器手势与页面焦点移动

在 Android TV 与 Google TV 界面中，滑动 iPhone 遥控器的触摸板区域，会实时映射为电视界面的焦点网格迁移：

```
[顶部标签导航 (For you / Apps)]
              ↕ (手势向下轻扫)
[巨幅内容海报聚焦 (Hero Banner)]
              ↕ (手势向下轻扫)
[应用启动底栏 (YouTube / VLC / 设置)]
```

| 1. 初始聚焦顶部导航 | 2. 向下滑动聚焦主推荐位 | 3. 继续下滑移动至应用卡片 |
| :---: | :---: | :---: |
| ![顶部导航](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/android_tv_launcher_home.png) | ![推荐位聚焦](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/android_tv_nav_content.png) | ![应用卡片聚焦](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/android_tv_nav_apps.png) |

* **轻触或按下中心 SELECT**：直接打开当前聚焦的卡片或播放视频。
* **按返回键 `<`**：退回上一级页面；
* **按电视图标 `Home`**：一键瞬时切回系统主页桌面。

---

### 2. Android TV App 控制看板与功能配置

直接在电视端打开应用，可以直观查看所有服务与连接状态，并自由定制按键策略：

| 控制中心主仪表板 | 按键注入模式选择 | Menu 键自定义映射 |
| :---: | :---: | :---: |
| ![主界面](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/android_tv_main_screen_ready.png) | ![注入模式](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/android_tv_injection_mode_dialog.png) | ![Menu键绑定](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/android_tv_menu_binding_dialog.png) |

* **三通道智能状态指示**：
  * **硬件通道**：检测是否存在 `/dev/input/event*` 直连驱动。
  * **ADB 通道**：检测本地 Dadb (`127.0.0.1:5555`) 守护进程是否连接成功。
  * **无障碍通道**：检测 `AtvAccessibilityService` 是否就绪。
* **按键注入优先策略**：
  * 支持自由切换：`仅本地 ADB`、`仅无障碍服务`、`ADB 优先 (无障碍兜底)`、`硬件优先` 等策略。
* **Menu 键自定义映射**：
  * 由于部分海外应用依赖 Android 物理 Menu 键，可将遥控器的「播放/暂停」、「Home」或「静音」键自由重映射为 Android 系统的 `KEYCODE_MENU`。

---

## 四、Android 模拟器联调与 Bridge 桥接操作指南

如果你手头没有物理 Android 电视，或者正在进行本地开发，可以使用 Android Studio 内置的 Android TV 模拟器（AVD）进行完全一致的遥控体验。

### 1. 模拟器网络通信架构

由于 Android 模拟器运行在宿主机内部的虚拟 NAT 子网（虚拟 IP 固定为 `10.0.2.15`），外界的局域网物理设备（如你的真实 iPhone）无法直接通过局域网广播发现模拟器。

本项目提供了专用的 **`scripts/bridge_emulator.py`** 桥接套件：

![模拟器桥接工作流](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/emulator_operation_flow.svg)

1. **ADB 端口透传**：将宿主机物理端口 (`49152`, `49153`, `49154`) 通过 `adb forward` 无缝穿透到模拟器内部。
2. **mDNS 代理广播**：宿主机接管 Bonjour 广播通道，代表模拟器向 Wi-Fi 物理局域网宣告 `_mediaremotetv._tcp` 与 `_companion-link._tcp` 服务。
3. **无感直连**：iPhone 在控制中心看到的设备即为运行在 Mac 上的 Android TV 模拟器。

---

### 2. 模拟器操作与桥接步骤

#### 第一步：启动 Android TV 模拟器
在 Android Studio Device Manager 中启动任意 Android TV / Google TV 镜像（推荐 Android 11+ / API 30+），或在终端中运行：
```bash
emulator -avd <你的电视镜像名称>
```
运行 `adb devices` 确认设备已挂载（通常为 `emulator-5554`）。

#### 第二步：安装并启动 TV 接收端
```bash
# 编译并安装 APK
./scripts/build_android.sh
adb -s emulator-5554 install -r android-tv/app/build/outputs/apk/debug/app-debug.apk

# 在模拟器中启动应用
adb -s emulator-5554 shell am start -n com.corvofeng.fakeatv/.MainActivity
```

#### 第三步：启动桥接服务 (Bridge)
```bash
# 启动后台桥接守护进程
python3 scripts/bridge_emulator.py start

# 或者以前台模式运行查看实时日志
python3 scripts/bridge_emulator.py start -f
```

* 常用管理命令：
  ```bash
  python3 scripts/bridge_emulator.py status    # 查看当前端口与 mDNS 广播状态
  python3 scripts/bridge_emulator.py logs -f   # 查看桥接日志
  python3 scripts/bridge_emulator.py stop      # 停止桥接
  ```

#### 第四步：在 iPhone 上配对并操控
1. 确保 iPhone 连接到与 Mac 相同的 Wi-Fi。
2. 下拉 iOS 控制中心，点击「Apple TV 遥控器」图标。
3. 在设备列表中选择 **`Android TV Emulator`**。
4. 输入默认配对码 **`1111`**，即可直接在大屏幕模拟器上遥控翻页！

---

## 五、macOS 原生菜单栏应用与使用指南

除了控制 Android TV，`atv-core` 还能将你的 Mac（尤其是 Mac mini / 客厅 HTPC）变成一台功能齐备的高性能 Apple TV。

```bash
# 一键打包原生 DMG 安装包
./scripts/build_dmg.sh
```

打开生成的 `build/AppleTVRemote-arm64.dmg`，将应用拖拽到 `/Applications` 即可：

* **常驻状态栏**：轻量级 Native Swift AppKit 架构，内存占用低至 18MB。
* **光标 / 方向键热切换**：
  * **鼠标光标模式**：集成动态人体工学加速度曲线，微移精准点选，轻划穿透多显示器。
  * **方向键模式**：精准触发 `↑` `↓` `←` `→` 与回车。
  * **切换方式**：按下遥控器侧边 **Siri / 语音键**，或者在 Web 控制面板点击一键切换，Mac 屏幕会自动弹出原生 Notification 横幅提示！
* **深度多媒体键映射**：
  * **单击 ⏯ 键**：播放 / 暂停（系统原生媒体键，适配 Bilibili、YouTube、Safari、Spotify、IINA 等）。
  * **双击 ⏯ 键**：下一曲（Next Track ⏭）。
  * **三击 ⏯ 键**：上一曲（Previous Track ⏮）。
  * **音量加/减/静音**：硬件级音量 HUD 联动。

---

## 六、macOS 浏览器实时控制面板 (Web Inspector)

在 macOS 上运行 `atv-cli` 或 `AppleTVRemote.app` 时，系统自动在本地启动精美的 Web 交互诊断控制台：

* **遥控交互控制台 (Core Web Inspector)**: 浏览器直接访问 👉 **`http://127.0.0.1:8765`**
* **系统运行日志与诊断 (Swift Host Service)**: 浏览器直接访问 👉 **`http://127.0.0.1:8766`**

### 1. macOS 浏览器界面实景配图 (Web Inspector Mockup)

![macOS 浏览器 Web Inspector 界面](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/mac_browser_inspector.png)

### 2. macOS Swift 宿主诊断与运行日志页面

![macOS Swift Host 运行日志界面](https://github.com/corvofeng/atv-core/releases/download/v1.0.0/mac_debug_web_page.png)

### 核心功能亮点：
* **触摸轨迹实时可视化**：实时 Canvas 渲染手指位移坐标、手势阶段（Began, Moved, Ended）与速度。
* **交互式虚拟遥控器**：无需拿着手机，在网页上直接点击按钮测试 Mac 或电视端响应。
* **灵敏度与加速度调节**：提供 `0.5x 慢速精准`、`1.0x 标准`、`1.5x 快速`、`2.2x 极速大屏` 预设，并支持自定义滑块与动力学算法开关。
* **毫秒级 SSE 协议事件流**：直观观察握手、解密与按键生命周期。

---

## 七、按键与手势映射速查表

| Apple TV 遥控器手势 / 按键 | Android TV (物理电视 / 模拟器) | macOS (Mac mini / MacBook) |
| :--- | :--- | :--- |
| **滑动触摸板 (方向模式)** | `KEYCODE_DPAD_UP` / `DOWN` / `LEFT` / `RIGHT` | 方向键 `↑` `↓` `←` `→` |
| **滑动触摸板 (鼠标模式)** | 模拟触摸拖拽 / 光标滑动 | 本地坐标累加器平滑光标 (`CGEvent`) |
| **轻触 / 点击 SELECT** | `KEYCODE_DPAD_CENTER` / 点击确认 | 回车键确认 (Enter) / 鼠标左键单击 |
| **返回键 (`<`)** | `GLOBAL_ACTION_BACK` / 返回上一级 | `Escape` 键 |
| **电视图标 (`Home`)** | `GLOBAL_ACTION_HOME` / 回到桌面 | 回到桌面 / 用户自定义快捷键 |
| **单击 ⏯** | `KEYCODE_MEDIA_PLAY_PAUSE` | 原生播放/暂停 (`NX_KEYTYPE_PLAY`) |
| **双击 ⏯** | 下一集 / 媒体快进 | 下一曲 (Next Track ⏭) |
| **三击 ⏯** | 上一集 / 媒体快退 | 上一曲 (Previous Track ⏮) |
| **音量加 / 减 (`+/-`)** | `KEYCODE_VOLUME_UP` / `DOWN` | 硬件主输出音量调节 + 原生 HUD |
| **静音键 (`Mute`)** | `KEYCODE_VOLUME_MUTE` | 系统一键静音 |
| **侧边 Siri 键** | 语音搜索 (`KEYCODE_SEARCH`) | 模式热切换 (鼠标 ⇄ 方向键) 或唤醒 Siri |
| **电源键 (`Power`)** | 电源休眠对话框 (`GLOBAL_ACTION_POWER_DIALOG`) | 屏幕睡眠 / 唤醒 |

---

## 八、常见问题排查 (FAQ)

### Q1: iPhone 控制中心找不到设备？
1. 检查 iPhone 与运行设备是否在同一个 Wi-Fi 网络下（避免一个连接 5G、另一个连接 2.4G 访客网络且开启了隔离开关）。
2. 在 Mac 终端运行 `dns-sd -B _mediaremotetv._tcp`，检查是否能正常发现本机的 Bonjour 广播。
3. 如果是 Android 模拟器，确保已运行 `python3 scripts/bridge_emulator.py start`。

### Q2: 电视上按键无反应，Live Log 提示 ADB 连接失败？
1. 请进入电视「开发者选项」，确认已打开「网络调试 (Network ADB)」与「USB 调试」。
2. 重启一次 TV 应用，密切留意电视屏幕中央，是否出现了 RSA 安全指纹授权弹窗。**必须勾选「始终允许」并点确定**。
3. 如果电视系统无原生弹窗，可以在电脑终端执行 `adb connect <电视IP>:5555` 进行首次电脑端信任。

### Q3: 为什么电视端安装后系统设置里找不到「无障碍」页面？
部分国内定制化 Android 电视系统阉割了原生设置菜单。可以在主界面点击「ACCESSIBILITY SETTINGS」弹窗查看 ADB 命令，或在电脑终端执行以下命令直接激活：
```bash
adb shell settings put secure enabled_accessibility_services com.corvofeng.fakeatv/.AtvAccessibilityService
adb shell settings put secure accessibility_enabled 1
```

### Q4: macOS 提示应用损坏或来自不受信任的开发者？
执行以下命令移除 Gatekeeper 隔离限制：
```bash
xattr -dr com.apple.quarantine /Applications/AppleTVRemote.app
```
或运行仓库提供的导入脚本信任自签名证书：
```bash
./scripts/import_certificate.sh Corvo_Development.p12
```

---

## 许可证 (License)

本项目采用 MIT 许可证开源。仅供技术学习与局域网设备互通研究使用。
