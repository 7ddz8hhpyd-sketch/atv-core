import AppKit

func renderIcon(size: CGFloat) -> NSImage {
    let img = NSImage(size: NSSize(width: size, height: size))
    img.lockFocus()

    guard let ctx = NSGraphicsContext.current?.cgContext else {
        img.unlockFocus()
        return img
    }

    let scale = size / 512.0

    // 1. App Icon Squircle Background
    let squircleRect = NSRect(x: 32 * scale, y: 32 * scale, width: 448 * scale, height: 448 * scale)
    let squirclePath = NSBezierPath(roundedRect: squircleRect, xRadius: 100 * scale, yRadius: 100 * scale)

    // Background Gradient (Obsidian to Deep Blue)
    let gradient = NSGradient(
        colors: [
            NSColor(red: 0.12, green: 0.14, blue: 0.19, alpha: 1.0),
            NSColor(red: 0.06, green: 0.07, blue: 0.10, alpha: 1.0)
        ]
    )
    gradient?.draw(in: squirclePath, angle: -45)

    // Squircle Border Glow
    squirclePath.lineWidth = 2.0 * scale
    NSColor(red: 0.25, green: 0.30, blue: 0.40, alpha: 0.5).setStroke()
    squirclePath.stroke()

    // 2. Wireless Waves / Signal above remote
    let waveCenter = CGPoint(x: 256 * scale, y: 410 * scale)
    for i in 1...3 {
        let r = CGFloat(i) * 16.0 * scale
        let arcPath = NSBezierPath()
        arcPath.appendArc(
            withCenter: waveCenter,
            radius: r,
            startAngle: 45,
            endAngle: 135,
            clockwise: false
        )
        arcPath.lineWidth = 2.5 * scale
        arcPath.lineCapStyle = .round
        let alpha = 0.9 - CGFloat(i) * 0.22
        NSColor(red: 0.44, green: 0.95, blue: 0.86, alpha: alpha).setStroke()
        arcPath.stroke()
    }

    // 3. Apple TV Remote Body (Silver / Space Aluminum)
    let remoteRect = NSRect(x: 176 * scale, y: 64 * scale, width: 160 * scale, height: 320 * scale)
    let remotePath = NSBezierPath(roundedRect: remoteRect, xRadius: 36 * scale, yRadius: 36 * scale)

    let remoteGradient = NSGradient(
        colors: [
            NSColor(red: 0.24, green: 0.27, blue: 0.33, alpha: 1.0),
            NSColor(red: 0.16, green: 0.18, blue: 0.22, alpha: 1.0)
        ]
    )
    remoteGradient?.draw(in: remotePath, angle: -90)

    remotePath.lineWidth = 1.5 * scale
    NSColor(red: 0.35, green: 0.40, blue: 0.50, alpha: 0.8).setStroke()
    remotePath.stroke()

    // 4. Clickpad / Touch Surface
    let padRect = NSRect(x: 196 * scale, y: 248 * scale, width: 120 * scale, height: 110 * scale)
    let padPath = NSBezierPath(roundedRect: padRect, xRadius: 28 * scale, yRadius: 28 * scale)

    // Touch surface glow gradient
    let padGradient = NSGradient(
        colors: [
            NSColor(red: 0.10, green: 0.45, blue: 0.42, alpha: 0.9),
            NSColor(red: 0.08, green: 0.28, blue: 0.32, alpha: 0.9)
        ]
    )
    padGradient?.draw(in: padPath, angle: -45)

    padPath.lineWidth = 2.0 * scale
    NSColor(red: 0.44, green: 0.95, blue: 0.86, alpha: 0.9).setStroke()
    padPath.stroke()

    // Center Click Dot
    let centerDot = NSBezierPath(ovalIn: NSRect(x: 244 * scale, y: 291 * scale, width: 24 * scale, height: 24 * scale))
    NSColor(red: 0.44, green: 0.95, blue: 0.86, alpha: 0.95).setFill()
    centerDot.fill()

    // 5. Remote Buttons (2x2 Grid)
    let btnColor = NSColor(red: 0.30, green: 0.34, blue: 0.42, alpha: 1.0)
    let btnSize = 36.0 * scale
    let btnY1 = 184.0 * scale
    let btnY2 = 132.0 * scale
    let btnX1 = 206.0 * scale
    let btnX2 = 270.0 * scale

    // Back / Menu Button
    let btnBack = NSBezierPath(ovalIn: NSRect(x: btnX1, y: btnY1, width: btnSize, height: btnSize))
    btnColor.setFill()
    btnBack.fill()

    // TV / Home Button
    let btnHome = NSBezierPath(ovalIn: NSRect(x: btnX2, y: btnY1, width: btnSize, height: btnSize))
    btnColor.setFill()
    btnHome.fill()

    // Play/Pause Button
    let btnPlay = NSBezierPath(ovalIn: NSRect(x: btnX1, y: btnY2, width: btnSize, height: btnSize))
    btnColor.setFill()
    btnPlay.fill()

    // Play icon symbol inside button
    let playPath = NSBezierPath()
    playPath.move(to: CGPoint(x: (btnX1 + 14 * scale), y: (btnY2 + 10 * scale)))
    playPath.line(to: CGPoint(x: (btnX1 + 25 * scale), y: (btnY2 + 18 * scale)))
    playPath.line(to: CGPoint(x: (btnX1 + 14 * scale), y: (btnY2 + 26 * scale)))
    playPath.close()
    NSColor.white.withAlphaComponent(0.85).setFill()
    playPath.fill()

    // Mute / Volume Button
    let btnMute = NSBezierPath(ovalIn: NSRect(x: btnX2, y: btnY2, width: btnSize, height: btnSize))
    btnColor.setFill()
    btnMute.fill()

    // 6. Mic / Siri Notch on right side
    let siriNotch = NSBezierPath(roundedRect: NSRect(x: 334 * scale, y: 220 * scale, width: 4 * scale, height: 32 * scale), xRadius: 2 * scale, yRadius: 2 * scale)
    NSColor(red: 0.50, green: 0.55, blue: 0.65, alpha: 0.9).setFill()
    siriNotch.fill()

    img.unlockFocus()
    return img
}

func savePNG(image: NSImage, path: String) {
    guard let tiff = image.tiffRepresentation,
          let rep = NSBitmapImageRep(data: tiff),
          let png = rep.representation(using: .png, properties: [:]) else {
        return
    }
    try? png.write(to: URL(fileURLWithPath: path))
}

let fm = FileManager.default
let scriptDir = URL(fileURLWithPath: CommandLine.arguments[0]).deletingLastPathComponent().path
let rootDir = URL(fileURLWithPath: scriptDir).deletingLastPathComponent().path
let iconsetDir = "\(rootDir)/mac-app/resources/AppIcon.iconset"
let icnsPath = "\(rootDir)/mac-app/resources/AppIcon.icns"

try? fm.createDirectory(atPath: iconsetDir, withIntermediateDirectories: true, attributes: nil)

let sizes: [(String, CGFloat)] = [
    ("icon_16x16.png", 16),
    ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32),
    ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128),
    ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256),
    ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512),
    ("icon_512x512@2x.png", 1024),
]

print("🎨 Generating Apple TV Remote App Icon assets...")
for (name, size) in sizes {
    let img = renderIcon(size: size)
    savePNG(image: img, path: "\(iconsetDir)/\(name)")
}

let process = Process()
process.executableURL = URL(fileURLWithPath: "/usr/bin/iconutil")
process.arguments = ["-c", "icns", iconsetDir, "-o", icnsPath]
try? process.run()
process.waitUntilExit()

if process.terminationStatus == 0 {
    print("✅ Successfully generated: \(icnsPath)")
    try? fm.removeItem(atPath: iconsetDir)
} else {
    print("⚠️ iconutil exited with status \(process.terminationStatus)")
}
