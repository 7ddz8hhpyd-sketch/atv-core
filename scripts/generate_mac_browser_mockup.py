import subprocess
import os

html_template = """<!doctype html>
<html>
<head>
<meta charset="utf-8">
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    background: radial-gradient(circle at 50% 30%, #1e1b4b 0%, #0f172a 60%, #020617 100%);
    padding: 30px;
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
  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }
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
  .address-bar .lock {
    color: #38bdf8;
    font-size: 11px;
  }
  .address-bar .host {
    color: #f1f5f9;
    font-weight: 500;
  }
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

# Copy dashboard.html
with open("crates/atv-core/src/dashboard.html", "r", encoding="utf-8") as f:
    dash = f.read()

with open("docs/images/dashboard_embed.html", "w", encoding="utf-8") as f:
    f.write(dash)

with open("docs/images/mac_browser_frame.html", "w", encoding="utf-8") as f:
    f.write(html_template)

chrome = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
url = "file://" + os.path.abspath("docs/images/mac_browser_frame.html")
out_png = "docs/images/mac_browser_inspector.png"

subprocess.run([
    chrome,
    "--headless",
    "--disable-gpu",
    "--window-size=1440,940",
    "--hide-scrollbars",
    f"--screenshot={out_png}",
    url
])

print("Generated:", out_png)
