import Foundation
import AppKit

final class ProcessManager {
    static let shared = ProcessManager()

    private var process: Process?
    private let logDir: URL
    private let logFile: URL

    private init() {
        let home = FileManager.default.homeDirectoryForCurrentUser
        self.logDir = home.appendingPathComponent("Library/Logs/AppleTVRemote")
        self.logFile = logDir.appendingPathComponent("atv-core.log")
        try? FileManager.default.createDirectory(at: logDir, withIntermediateDirectories: true, attributes: nil)
    }

    /// Resolve the path to the atv-cli executable
    func findAtvCli() -> URL? {
        let fm = FileManager.default

        // 1. Inside App Bundle Contents/MacOS/atv-cli
        let bundleBinary = Bundle.main.bundleURL.appendingPathComponent("Contents/MacOS/atv-cli")
        if fm.isExecutableFile(atPath: bundleBinary.path) {
            return bundleBinary
        }

        // 2. Sibling to executable
        if let execDir = Bundle.main.executableURL?.deletingLastPathComponent() {
            let sibling = execDir.appendingPathComponent("atv-cli")
            if fm.isExecutableFile(atPath: sibling.path) {
                return sibling
            }
        }

        // 3. Workspace dev build locations
        let candidates = [
            fm.currentDirectoryPath + "/target/release/atv-cli",
            fm.currentDirectoryPath + "/target/debug/atv-cli",
            (homePath() ?? "") + "/.local/bin/atv-cli",
            "/usr/local/bin/atv-cli",
            "/opt/homebrew/bin/atv-cli"
        ]

        for p in candidates {
            if fm.isExecutableFile(atPath: p) {
                return URL(fileURLWithPath: p)
            }
        }

        return nil
    }

    private func homePath() -> String? {
        return ProcessInfo.processInfo.environment["HOME"]
    }

    /// Check if the service is currently running and responding
    func checkServiceRunning(completion: @escaping (Bool) -> Void) {
        ApiClient.shared.fetchStatus { status in
            completion(status != nil)
        }
    }

    /// Start the atv-cli core server if not already running
    func startService(completion: @escaping (Bool) -> Void) {
        // First check if a server instance is already running
        ApiClient.shared.fetchStatus { [weak self] status in
            if status != nil {
                print("ℹ️ atv-core server is already running.")
                completion(true)
                return
            }

            guard let binary = self?.findAtvCli() else {
                print("❌ Could not find atv-cli binary.")
                completion(false)
                return
            }

            self?.launchProcess(binaryUrl: binary, completion: completion)
        }
    }

    private func launchProcess(binaryUrl: URL, completion: @escaping (Bool) -> Void) {
        let p = Process()
        p.executableURL = binaryUrl
        p.arguments = ["--target", "mac", "--pin", "1111", "--no-prompt"]

        // Create log file handle
        let fm = FileManager.default
        if !fm.fileExists(atPath: logFile.path) {
            fm.createFile(atPath: logFile.path, contents: nil, attributes: nil)
        }

        if let fileHandle = try? FileHandle(forWritingTo: logFile) {
            fileHandle.seekToEndOfFile()
            p.standardOutput = fileHandle
            p.standardError = fileHandle
        }

        p.terminationHandler = { [weak self] proc in
            print("⚠️ atv-core process terminated with exit code \(proc.terminationStatus)")
            self?.process = nil
        }

        do {
            try p.run()
            self.process = p
            print("🚀 Started atv-core (PID \(p.processIdentifier)) from \(binaryUrl.path)")

            // Wait briefly for server startup, then verify
            DispatchQueue.global().asyncAfter(deadline: .now() + 0.8) {
                ApiClient.shared.fetchStatus { status in
                    completion(status != nil)
                }
            }
        } catch {
            print("❌ Failed to start atv-core process: \(error)")
            completion(false)
        }
    }

    /// Stop the atv-cli service (terminating child process and any external instance)
    func stopService(completion: (() -> Void)? = nil) {
        if let p = process, p.isRunning {
            print("🛑 Terminating atv-core child process...")
            p.terminate()
            p.waitUntilExit()
            process = nil
        }

        // Also terminate any standalone/external atv-cli process to release ports
        let killTask = Process()
        killTask.executableURL = URL(fileURLWithPath: "/usr/bin/pkill")
        killTask.arguments = ["-f", "atv-cli"]
        try? killTask.run()
        killTask.waitUntilExit()

        DispatchQueue.global().asyncAfter(deadline: .now() + 0.3) {
            completion?()
        }
    }

    /// Restart the service cleanly
    func restartService(completion: @escaping (Bool) -> Void) {
        print("🔄 Restarting atv-core service...")
        stopService { [weak self] in
            DispatchQueue.global().asyncAfter(deadline: .now() + 0.5) {
                self?.startService(completion: completion)
            }
        }
    }

    /// Convenience wrapper for backward compatibility
    func startCoreIfNeeded(completion: @escaping (Bool) -> Void) {
        startService(completion: completion)
    }

    func stopCore() {
        stopService(completion: nil)
    }
}
