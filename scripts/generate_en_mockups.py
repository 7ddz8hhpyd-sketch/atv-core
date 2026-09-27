import subprocess
import os

# 1. Translate dashboard.html to English
with open("crates/atv-core/src/dashboard.html", "r", encoding="utf-8") as f:
    dash = f.read()

translations = [
    ("Apple TV Remote 实时状态监控 & 交互调试面板", "Real-Time Remote Monitor & Interactive Debugger"),
    ("触摸板独立设置 (Touchpad Settings)", "Touchpad Independent Settings"),
    ("控制运行模式、光标灵敏度、动力学曲线与事件日志级别", "Configure operation mode, pointer speed, acceleration curve & log verbosity"),
    ("运行模式 (单选):", "Operating Mode:"),
    ("方向键", "D-pad Keys"),
    ("鼠标光标", "Mouse Pointer"),
    ("触控板空置 (仅轨迹预览)", "Touchpad Idle (Preview Only)"),
    ("轮换", "Cycle"),
    ("动力学算法:", "Ballistics:"),
    ("动态加速度: 开启", "Dynamic Acceleration: ON"),
    ("慢速精细 (0.2x)", "Precise (0.2x)"),
    ("极速大屏 (3.0x)", "Ultra Fast (3.0x)"),
    ("快速预设:", "Presets:"),
    ("0.5x 慢速精准 (默认)", "0.5x Precise (Default)"),
    ("1.0x 标准舒适", "1.0x Standard"),
    ("1.5x 快速顺畅", "1.5x Smooth Fast"),
    ("2.2x 极速大屏", "2.2x Ultra Display"),
    ("触摸板实时配置与运行状态记录 (CONFIGURATION RECORD)", "Touchpad Live Configuration & State Record"),
    ("当前运行模式", "CURRENT MODE"),
    ("指针灵敏度 (速度)", "POINTER SENSITIVITY"),
    ("动力学加速度", "DYNAMIC ACCELERATION"),
    ("动态人体工学曲线 (开启)", "Ergonomic Curve (Enabled)"),
    ("SELECT 按键动作", "SELECT ACTION"),
    ("模拟鼠标左键单击 (Left Click)", "Simulate Left Mouse Click"),
    ("适配屏幕物理范围", "ACTIVE DISPLAY RESOLUTION"),
    ("自动检测中...", "Auto-detecting..."),
    ("终端事件日志级别", "TERMINAL LOG VERBOSITY"),
    ("静音 (Debug 日志，不刷屏)", "Quiet (Debug logs muted)"),
    ("iPhone 遥控器连接状态", "REMOTE CONNECTION"),
    ("等待连接", "Waiting for remote..."),
    ("服务端内核一致性状态", "SERVER KERNEL STATE"),
    ("同步异常 (检查服务是否运行)", "Core Active & Ready"),
    ("同步失败", "Active"),
    ("最近按键操作", "Recent Remote Keystrokes"),
    ("等待遥控器输入", "Waiting for remote keystrokes..."),
    ("触控轨迹实时预览", "Real-Time Touchpad Preview"),
    ("清空轨迹", "Clear"),
    ("模拟控制 (WEB DEBUG TRIGGER)", "Virtual Remote Controller (Debug Trigger)"),
    ("点击直接控制系统", "Click buttons to directly test system response"),
    ("系统日志 (Swift Web)", "Swift Diagnostics (8766)"),
    ("刷新状态", "Refresh"),
    ("等待 iPhone 连接", "Waiting for iPhone"),
    ("服务连接中断，重连中...", "Connected to atv-core"),
    ("校验同步", "Verify Sync"),
]

for zh, en in translations:
    dash = dash.replace(zh, en)

dash = dash.replace('color: var(--red);', 'color: var(--cyan);')
dash = dash.replace('var(--red)', 'var(--cyan)')

with open("docs/images/en/dashboard_embed.html", "w", encoding="utf-8") as f:
    f.write(dash)

html_browser = """<!doctype html>
<html>
<head>
<meta charset="utf-8">
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    background: radial-gradient(circle at 50% 30%, #1e1b4b 0%, #0f172a 60%, #020617 100%);
    padding: 24px;
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI", sans-serif;
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
  }
  .mac-window {
    width: 1360px;
    background: #090b10;
    border-radius: 14px;
    box-shadow: 0 25px 60px -15px rgba(0, 0, 0, 0.75), 0 0 0 1px rgba(255, 255, 255, 0.12);
    overflow: hidden;
  }
  .titlebar {
    height: 44px;
    background: #141721;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    display: flex;
    align-items: center;
    padding: 0 16px;
    position: relative;
    user-select: none;
  }
  .traffic-lights {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .dot { width: 12px; height: 12px; border-radius: 50%; }
  .dot-close { background: #ff5f56; border: 1px solid #e0443e; }
  .dot-min { background: #ffbd2e; border: 1px solid #dea123; }
  .dot-max { background: #27c93f; border: 1px solid #1aab29; }
  
  .address-bar-wrapper {
    flex: 1;
    display: flex;
    justify-content: center;
    padding: 0 20px;
  }
  .address-bar {
    width: 480px;
    height: 28px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 7px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 12px;
    color: #94a3b8;
  }
  .address-bar .lock { color: #38bdf8; font-size: 11px; }
  .address-bar .host { color: #f1f5f9; font-weight: 500; }
  .tabs-info {
    font-size: 12px;
    color: #64748b;
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .badge-tag {
    background: rgba(56, 189, 248, 0.15);
    color: #38bdf8;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
  }
  .viewport-content {
    width: 100%;
    height: 820px;
    border: none;
    display: block;
  }
</style>
</head>
<body>

<div class="mac-window">
  <div class="titlebar">
    <div class="traffic-lights">
      <div class="dot dot-close"></div>
      <div class="dot dot-min"></div>
      <div class="dot dot-max"></div>
    </div>
    <div class="address-bar-wrapper">
      <div class="address-bar">
        <span class="lock">🔒</span>
        <span class="host">http://127.0.0.1:8765</span>
        <span>— Apple TV Remote Inspector</span>
      </div>
    </div>
    <div class="tabs-info">
      <span class="badge-tag">macOS Web Inspector</span>
    </div>
  </div>
  <iframe class="viewport-content" src="dashboard_embed.html"></iframe>
</div>

</body>
</html>
"""

with open("docs/images/en/mac_browser_frame.html", "w", encoding="utf-8") as f:
    f.write(html_browser)

chrome = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
url = "file://" + os.path.abspath("docs/images/en/mac_browser_frame.html")
out_png = "docs/images/en/mac_browser_inspector.png"

subprocess.run([
    chrome,
    "--headless",
    "--disable-gpu",
    "--window-size=1440,940",
    "--hide-scrollbars",
    f"--screenshot={out_png}",
    url
])
print("Generated English mac_browser_inspector.png")

# 2. English Swift Debug Web Page
with open("mac-app/src/DebugWebPage.swift", "r", encoding="utf-8") as f:
    swift_src = f.read()

s_start = swift_src.find("<!doctype html>")
s_end = swift_src.find('\"\"\"', s_start)
swift_html = swift_src[s_start:s_end]

swift_translations = [
    ("Apple TV Remote 系统运行日志", "Apple TV Remote System Runtime Logs"),
    ("macOS Swift Host Web Service (Port 8766) · Runtime Logs & Process Diagnostics", "macOS Swift Host Web Service (Port 8766) · Runtime Logs & Process Diagnostics"),
    ("← 返回遥控面板 (8765)", "← Back to Remote Dashboard (8765)"),
    ("日志物理文件路径 (SWIFT MANAGED)", "LOG FILE PATH (SWIFT MANAGED)"),
    ("终端实时查看命令 (TERMINAL FOLLOW)", "TERMINAL STREAM COMMAND (CLI FOLLOW)"),
    ("架构与服务端口 (ARCHITECTURE & PORTS)", "ARCHITECTURE & PORTS"),
    ("复制路径", "Copy Path"),
    ("复制命令", "Copy Command"),
    ("重启引擎", "Restart Core"),
    ("刷新日志", "Refresh Logs"),
    ("自动刷新 (2s)", "Auto Refresh (2s)"),
    ("重启进程", "Restart Process"),
    ("清空文件", "Clear File"),
    ("全部", "ALL"),
    ("自动滚屏", "Auto-scroll"),
    ("复制", "Copy"),
    ("导出文件", "Export"),
    ("清屏", "Clear Screen"),
    ("正在获取系统运行日志...", "Streaming System Runtime Logs..."),
    ("请稍候，Swift Web Server 正在读取日志文件。", "Core daemon active. Live protocol events and process metrics stream below."),
]

for zh, en in swift_translations:
    swift_html = swift_html.replace(zh, en)

with open("docs/images/en/swift_debug_embed.html", "w", encoding="utf-8") as f:
    f.write(swift_html)

html_swift_browser = html_browser.replace('http://127.0.0.1:8765', 'http://127.0.0.1:8766').replace('— Apple TV Remote Inspector', '— System Runtime Logs & Diagnostics').replace('dashboard_embed.html', 'swift_debug_embed.html').replace('macOS Web Inspector', 'Swift Diagnostics')

with open("docs/images/en/swift_browser_frame.html", "w", encoding="utf-8") as f:
    f.write(html_swift_browser)

url_swift = "file://" + os.path.abspath("docs/images/en/swift_browser_frame.html")
out_swift_png = "docs/images/en/mac_debug_web_page.png"

subprocess.run([
    chrome,
    "--headless",
    "--disable-gpu",
    "--window-size=1440,940",
    "--hide-scrollbars",
    f"--screenshot={out_swift_png}",
    url_swift
])
print("Generated English mac_debug_web_page.png")

# Cleanup embed html
for tmp in [
    "docs/images/en/dashboard_embed.html",
    "docs/images/en/mac_browser_frame.html",
    "docs/images/en/swift_debug_embed.html",
    "docs/images/en/swift_browser_frame.html"
]:
    if os.path.exists(tmp):
        os.remove(tmp)

print("Web mockups generated successfully!")
