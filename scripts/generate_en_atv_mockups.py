import subprocess
import os

os.makedirs("docs/images/en", exist_ok=True)
chrome = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"

# 1. Android TV Main Screen Ready (EN)
html_main = """<!doctype html>
<html>
<head>
<meta charset="utf-8">
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    width: 1280px;
    height: 720px;
    background: #121212;
    color: #f1f5f9;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    padding: 32px 48px;
    overflow: hidden;
  }
  h1 { font-size: 26px; font-weight: 700; color: #ffffff; letter-spacing: -0.02em; }
  .subtitle { font-size: 13px; color: #94a3b8; margin-top: 4px; margin-bottom: 24px; }
  .grid { display: flex; gap: 24px; height: 580px; }
  .panel {
    flex: 1;
    background: #1e1e1e;
    border-radius: 8px;
    padding: 24px 28px;
    display: flex;
    flex-direction: column;
  }
  .pin-title { font-size: 22px; font-weight: 700; color: #4caf50; margin-bottom: 12px; }
  .info-line { font-size: 14px; color: #e2e8f0; margin-bottom: 10px; }
  .status-green { color: #4caf50; font-weight: 600; }
  .mode-blue { color: #38bdf8; font-weight: 600; }
  .channels { color: #cbd5e1; font-size: 13px; margin-top: 6px; line-height: 1.5; white-space: pre-line; }
  .btn-row { display: flex; gap: 12px; margin-top: 20px; flex-wrap: wrap; }
  .btn {
    background: #334155;
    color: #f8fafc;
    border: none;
    border-radius: 4px;
    padding: 10px 18px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
  }
  .log-title { font-size: 18px; font-weight: 700; color: #38bdf8; margin-bottom: 14px; }
  .log-box {
    flex: 1;
    background: #141414;
    border-radius: 4px;
    padding: 14px;
    font-family: ui-monospace, monospace;
    font-size: 11px;
    color: #cbd5e1;
    line-height: 1.6;
    overflow: hidden;
  }
  .log-hl { color: #4caf50; font-weight: 600; }
</style>
</head>
<body>
  <h1>Apple TV Remote Receiver</h1>
  <div class="subtitle">Control your Android TV directly using an iPhone / iPad Control Center Remote</div>

  <div class="grid">
    <div class="panel">
      <div class="pin-title">Pairing PIN: 1111</div>
      <div class="info-line">IP: 10.0.2.15 &nbsp;&nbsp; ID: 0E:30:68:D8:70:AB</div>
      <div class="info-line status-green">Android TV Device: Running</div>
      <div class="info-line status-green">Accessibility Service: Enabled (Fallback ready)</div>
      <div class="info-line">Menu Key Binding: Play/Pause Button (play_pause)</div>
      <div class="info-line mode-blue">Key Injection Mode: Local ADB Only (127.0.0.1:5555)</div>
      <div class="channels">Channels: Hardware [None] | ADB [● Connected] | A11y [● Ready]
Active Engine: Local ADB Daemon</div>

      <div class="btn-row">
        <button class="btn">STOP SERVER</button>
        <button class="btn">ACCESSIBILITY SETTINGS</button>
      </div>
      <div class="btn-row" style="margin-top: 10px;">
        <button class="btn">SWITCH INJECTION MODE</button>
        <button class="btn">BIND MENU KEY</button>
      </div>
    </div>

    <div class="panel">
      <div class="log-title">Live Event Log</div>
      <div class="log-box">
[Event] ClientConnected: 10.0.2.2:51234<br>
[Event] CompanionLink: TLS handshake completed<br>
[Event] SRP Pairing: Success (PIN verified 1111)<br>
<span class="log-hl">[Button] dpad_down [Local ADB]</span><br>
[Event] ControlMessage: {"_i": "_touchMove", "_x": 182, "dy": 35.2}<br>
<span class="log-hl">[Button] dpad_right [Local ADB]</span><br>
<span class="log-hl">[Button] select [Local ADB]</span><br>
[Event] ActiveNowPlaying: YouTube (Streaming)<br>
<span class="log-hl">[Button] volume_up [Local ADB]</span><br>
[Event] VolumeLevel: 65% synced to iOS Remote<br>
[Event] SessionActive: Direct zero-lag control<br>
      </div>
    </div>
  </div>
</body>
</html>
"""

with open("docs/images/en/atv_main.html", "w") as f:
    f.write(html_main)

subprocess.run([
    chrome, "--headless", "--disable-gpu", "--window-size=1280,720", "--hide-scrollbars",
    "--screenshot=docs/images/en/android_tv_main_screen_ready.png",
    "file://" + os.path.abspath("docs/images/en/atv_main.html")
])

# 2. Android TV Injection Mode Dialog (EN)
html_dlg1 = """<!doctype html>
<html>
<head>
<meta charset="utf-8">
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    width: 1280px; height: 720px; background: rgba(0,0,0,0.7);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    display: flex; justify-content: center; align-items: center;
  }
  .dialog {
    width: 560px; background: #242424; border-radius: 8px; padding: 24px 28px;
    box-shadow: 0 20px 50px rgba(0,0,0,0.8); color: #f8fafc;
  }
  h2 { font-size: 20px; font-weight: 700; margin-bottom: 20px; color: #ffffff; }
  .item { display: flex; align-items: center; gap: 14px; padding: 12px 14px; border-radius: 6px; font-size: 15px; margin-bottom: 4px; }
  .item.selected { background: rgba(56, 189, 248, 0.15); color: #38bdf8; font-weight: 600; }
  .radio { width: 18px; height: 18px; border-radius: 50%; border: 2px solid #64748b; display: flex; align-items: center; justify-content: center; }
  .selected .radio { border-color: #38bdf8; }
  .radio-inner { width: 8px; height: 8px; border-radius: 50%; background: #38bdf8; }
  .footer { display: flex; justify-content: flex-end; margin-top: 24px; }
  .cancel-btn { background: none; border: none; color: #38bdf8; font-size: 14px; font-weight: 700; cursor: pointer; padding: 8px 16px; }
</style>
</head>
<body>
  <div class="dialog">
    <h2>Select Key Injection Strategy (Priority)</h2>
    <div class="item selected">
      <div class="radio"><div class="radio-inner"></div></div>
      <div>Local ADB Only (127.0.0.1:5555)</div>
    </div>
    <div class="item">
      <div class="radio"></div>
      <div>ADB Preferred (Accessibility Fallback)</div>
    </div>
    <div class="item">
      <div class="radio"></div>
      <div>Accessibility Service Only</div>
    </div>
    <div class="item">
      <div class="radio"></div>
      <div>Hardware Input Driver Preferred (/dev/input)</div>
    </div>
    <div class="footer">
      <button class="cancel-btn">CANCEL</button>
    </div>
  </div>
</body>
</html>
"""

with open("docs/images/en/atv_dlg1.html", "w") as f:
    f.write(html_dlg1)

subprocess.run([
    chrome, "--headless", "--disable-gpu", "--window-size=1280,720", "--hide-scrollbars",
    "--screenshot=docs/images/en/android_tv_injection_mode_dialog.png",
    "file://" + os.path.abspath("docs/images/en/atv_dlg1.html")
])

# 3. Android TV Menu Binding Dialog (EN)
html_dlg2 = """<!doctype html>
<html>
<head>
<meta charset="utf-8">
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    width: 1280px; height: 720px; background: rgba(0,0,0,0.7);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    display: flex; justify-content: center; align-items: center;
  }
  .dialog {
    width: 580px; background: #242424; border-radius: 8px; padding: 24px 28px;
    box-shadow: 0 20px 50px rgba(0,0,0,0.8); color: #f8fafc;
  }
  h2 { font-size: 20px; font-weight: 700; margin-bottom: 20px; color: #ffffff; }
  .item { display: flex; align-items: center; gap: 14px; padding: 12px 14px; border-radius: 6px; font-size: 15px; margin-bottom: 4px; }
  .item.selected { background: rgba(56, 189, 248, 0.15); color: #38bdf8; font-weight: 600; }
  .radio { width: 18px; height: 18px; border-radius: 50%; border: 2px solid #64748b; display: flex; align-items: center; justify-content: center; }
  .selected .radio { border-color: #38bdf8; }
  .radio-inner { width: 8px; height: 8px; border-radius: 50%; background: #38bdf8; }
  .footer { display: flex; justify-content: flex-end; margin-top: 24px; }
  .cancel-btn { background: none; border: none; color: #38bdf8; font-size: 14px; font-weight: 700; cursor: pointer; padding: 8px 16px; }
</style>
</head>
<body>
  <div class="dialog">
    <h2>Bind Remote Button to Android Menu Key</h2>
    <div class="item">
      <div class="radio"></div>
      <div>None (Default: preserve original button function)</div>
    </div>
    <div class="item selected">
      <div class="radio"><div class="radio-inner"></div></div>
      <div>Play/Pause Button (play_pause, Recommended)</div>
    </div>
    <div class="item">
      <div class="radio"></div>
      <div>TV / Home Icon Button (home)</div>
    </div>
    <div class="item">
      <div class="radio"></div>
      <div>Mute Button (mute)</div>
    </div>
    <div class="item">
      <div class="radio"></div>
      <div>Back Button (menu)</div>
    </div>
    <div class="footer">
      <button class="cancel-btn">CANCEL</button>
    </div>
  </div>
</body>
</html>
"""

with open("docs/images/en/atv_dlg2.html", "w") as f:
    f.write(html_dlg2)

subprocess.run([
    chrome, "--headless", "--disable-gpu", "--window-size=1280,720", "--hide-scrollbars",
    "--screenshot=docs/images/en/android_tv_menu_binding_dialog.png",
    "file://" + os.path.abspath("docs/images/en/atv_dlg2.html")
])

# Copy language-neutral screenshots into docs/images/en/
for img in [
    "android_tv_launcher_home.png",
    "android_tv_nav_content.png",
    "android_tv_nav_apps.png",
    "android_tv_accessibility_service_list.png",
    "android_tv_permission_dialog.png"
]:
    src = os.path.join("docs/images", img)
    dst = os.path.join("docs/images/en", img)
    if os.path.exists(src):
        shutil.copy2(src, dst)

# Also copy all Chinese assets into docs/images/zh/
for img in os.listdir("docs/images"):
    src = os.path.join("docs/images", img)
    if os.path.isfile(src):
        shutil.copy2(src, os.path.join("docs/images/zh", img))

# Cleanup temporary html files
for tmp in ["docs/images/en/atv_main.html", "docs/images/en/atv_dlg1.html", "docs/images/en/atv_dlg2.html"]:
    if os.path.exists(tmp):
        os.remove(tmp)

print("All English TV and Dialog mockups generated!")
