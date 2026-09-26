//! atv-cli: Apple TV Remote simulator and controller for macOS and Android TV.

use std::process::exit;
use std::sync::Arc;

use atv_core::{AtvConfig, AtvDelegate, AtvServer};

#[cfg(target_os = "macos")]
mod macos;
mod android_adb;
mod print;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {
    Mac,
    Android,
    Print,
}

fn usage() -> ! {
    eprintln!(
        "usage: atv-cli [OPTIONS]\n\
         \n\
         Starts a fake Apple TV server (Bonjour, SRP PIN pairing, Companion Link)\n\
         that accepts control from iPhone's Control Center \"Apple TV Remote\".\n\
         \n\
         Options:\n\
           --target <mac|android|print>   Target device to control (default: mac on macOS, else print)\n\
           --android-host <IP>            Android TV IP address (required when --target android)\n\
           --android-port <PORT>          Android TV ADB port (default: 5555)\n\
           --name <NAME>                  Device name shown in iOS Remote (default: Computer Name on macOS, else \"Mac Remote\")\n\
           --device-id <MAC>              Hardware / MAC device ID (default: auto-detected hardware MAC)\n\
           --server-id <UUID>             Unique Server UUID (default: auto-detected hardware UUID)\n\
           --pin <PIN>                    4-digit pairing PIN (default: 1111)\n\
           --ip <IPV4>                    Local IPv4 to advertise (auto-detected if omitted)\n\
           --mouse                        Initial mode: Trackpad mouse (default on macOS)\n\
           --direction, --dpad            Initial mode: Direction keys (D-pad) (default on other targets)\n\
           --speed <FLOAT>                Initial mouse speed multiplier (default: 0.5)\n\
           --verbose-events               Enable verbose terminal logging for buttons and events (default: quiet debug)\n\
           --debug, -d                    Enable debug-level logging for all protocol packets\n\
           --ui-port <PORT>               Debug Web UI port (default: 8765)\n\
           --no-ui                        Disable Debug Web UI\n\
           --open                         Automatically open Debug UI in default browser\n\
           -h, --help                     Show this help message"
    );
    exit(2);
}

struct CliArgs {
    config: AtvConfig,
    target: Target,
    android_host: Option<String>,
    android_port: u16,
    open_browser: bool,
    verbose_events: bool,
    prompt_accessibility: bool,
}

fn parse_args() -> CliArgs {
    let mut config = AtvConfig::default();
    #[cfg(target_os = "macos")]
    let mut target = Target::Mac;
    #[cfg(not(target_os = "macos"))]
    let mut target = Target::Print;

    let mut android_host = None;
    let mut android_port = 5555;
    let mut open_browser = false;
    let mut verbose_events = false;
    let mut mouse_mode_explicit = false;
    let mut mouse_speed_explicit = false;
    let mut prompt_accessibility = true;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--target" => {
                let t = args.next().unwrap_or_else(|| usage());
                match t.as_str() {
                    "mac" => target = Target::Mac,
                    "android" => target = Target::Android,
                    "print" | "observe" => target = Target::Print,
                    _ => usage(),
                }
            }
            "--android-host" => {
                android_host = Some(args.next().unwrap_or_else(|| usage()));
            }
            "--android-port" => {
                android_port = args
                    .next()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or_else(|| usage());
            }
            "--name" => config.name = args.next().unwrap_or_else(|| usage()),
            "--device-id" => config.device_id = Some(args.next().unwrap_or_else(|| usage())),
            "--server-id" => config.server_identifier = Some(args.next().unwrap_or_else(|| usage())),
            "--pin" => {
                let pin: u32 = args
                    .next()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or_else(|| usage());
                if pin > 9999 {
                    usage();
                }
                config.pin = pin;
            }
            "--ip" => {
                config.ip = Some(
                    args.next()
                        .and_then(|ip| ip.parse().ok())
                        .unwrap_or_else(|| usage()),
                );
            }
            "--mouse" => {
                config.mouse_mode = true;
                mouse_mode_explicit = true;
            }
            "--direction" | "--dpad" => {
                config.mouse_mode = false;
                mouse_mode_explicit = true;
            }
            "--speed" => {
                let sp: f64 = args
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| usage());
                config.mouse_speed = sp;
                mouse_speed_explicit = true;
            }
            "--verbose-events" | "--verbose" => verbose_events = true,
            "--debug" | "-d" => {}
            "--no-prompt" => prompt_accessibility = false,
            "--ui-port" => {
                let port: u16 = args
                    .next()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or_else(|| usage());
                config.ui_port = Some(port);
            }
            "--no-ui" => config.ui_port = None,
            "--open" => open_browser = true,
            "-h" | "--help" => usage(),
            _ => usage(),
        }
    }

    if mouse_speed_explicit {
        atv_core::UserSettings::update_speed(config.mouse_speed);
    }

    if !mouse_mode_explicit {
        #[cfg(target_os = "macos")]
        if target == Target::Mac {
            let settings = atv_core::UserSettings::load();
            config.mouse_mode = settings.mouse_mode.unwrap_or(true);
        }
    }

    if target == Target::Android && android_host.is_none() {
        eprintln!("Error: --target android requires --android-host <IP>");
        usage();
    }

    CliArgs {
        config,
        target,
        android_host,
        android_port,
        open_browser,
        verbose_events,
        prompt_accessibility,
    }
}

#[tokio::main]
async fn main() {
    let has_debug_flag = std::env::args().any(|a| a == "--debug" || a == "-d");
    let default_filter = if has_debug_flag {
        "atv_core=debug,atv_cli=debug"
    } else {
        "atv_core=info,atv_cli=info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| default_filter.into()),
        )
        .init();

    let args = parse_args();
    let mouse_mode = Arc::new(std::sync::atomic::AtomicBool::new(args.config.mouse_mode));
    let open_browser = args.open_browser;

    let delegate: Arc<dyn AtvDelegate> = match args.target {
        Target::Mac => {
            #[cfg(target_os = "macos")]
            {
                let settings = atv_core::UserSettings::load();
                let accel = settings.mouse_accel.unwrap_or(true);
                let verbose = if args.verbose_events {
                    true
                } else {
                    settings.verbose_events.unwrap_or(false)
                };
                Arc::new(macos::MacDelegate::new_with_full_settings(
                    mouse_mode.clone(),
                    args.config.mouse_speed,
                    accel,
                    verbose,
                    args.prompt_accessibility,
                ))
            }
            #[cfg(not(target_os = "macos"))]
            {
                eprintln!("Warning: Target 'mac' is only supported on macOS; falling back to print");
                Arc::new(print::PrintDelegate)
            }
        }
        Target::Android => {
            let host = args.android_host.expect("checked above");
            println!("Target      : Android TV at {host}:{}", args.android_port);
            Arc::new(android_adb::AndroidAdbDelegate::new(
                host,
                args.android_port,
                mouse_mode.load(std::sync::atomic::Ordering::SeqCst),
            ))
        }
        Target::Print => {
            println!("Target      : observe-only (printing events to stdout)");
            Arc::new(print::PrintDelegate)
        }
    };

    let server = match AtvServer::start_with_mouse_mode(args.config, delegate.clone(), mouse_mode.clone()).await {
        Ok(server) => server,
        Err(e) => {
            eprintln!("failed to start server: {e}");
            exit(1);
        }
    };

    let identity = server.identity();
    println!("Device Name : \"{}\"", identity.name);
    println!("Device ID   : {}", identity.device_id);
    println!("Server UUID : {}", identity.server_identifier);
    println!("PIN Code    : {:04}", server.config().pin);

    let (init_speed, init_accel, init_verbose) = delegate.get_touchpad_settings();
    let initial_mode_str = if mouse_mode.load(std::sync::atomic::Ordering::SeqCst) {
        "🖱️  Mouse Cursor (鼠标光标模式)"
    } else {
        "◀▲▼▶ Direction Keys (上下左右模式)"
    };
    println!("Input Mode  : {initial_mode_str}");
    println!(
        "Touchpad    : Speed={:.2}x, Accel={}, VerboseEvents={}",
        init_speed,
        if init_accel { "Enabled" } else { "Disabled" },
        if init_verbose { "Enabled (Info)" } else { "Quiet (Debug)" }
    );
    println!("Interactive : 'm' toggle mode, 's <speed>' adjust speed, 'v' toggle verbose events");

    // Spawn terminal input listener for dynamic mode, speed, and verbose logging switching
    let mouse_mode_terminal = mouse_mode.clone();
    let delegate_terminal = delegate.clone();
    let inspector_terminal = server.inspector();
    tokio::spawn(async move {
        use tokio::io::AsyncBufReadExt;
        let stdin = tokio::io::stdin();
        let mut reader = tokio::io::BufReader::new(stdin).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case("m") || trimmed.eq_ignore_ascii_case("mode") {
                let current = mouse_mode_terminal.load(std::sync::atomic::Ordering::SeqCst);
                let new_mode = !current;
                mouse_mode_terminal.store(new_mode, std::sync::atomic::Ordering::SeqCst);
                delegate_terminal.on_mode_changed(new_mode);
                inspector_terminal.emit(
                    "mode_changed",
                    &format!("{{\"mouse_mode\":{}}}", new_mode),
                );
                let desc = if new_mode {
                    "🖱️  Mouse Cursor (鼠标光标模式)"
                } else {
                    "◀▲▼▶ Direction Keys (上下左右模式)"
                };
                println!("\n🔄 [模式切换] 当前已切换为: {desc}\n");
            } else if trimmed.starts_with("s ") || trimmed.starts_with("speed ") {
                let val_str = trimmed.split_whitespace().nth(1).unwrap_or("");
                if let Ok(val) = val_str.parse::<f64>() {
                    let (_, accel, verbose) = delegate_terminal.get_touchpad_settings();
                    let new_speed = val.clamp(0.1, 10.0);
                    delegate_terminal.on_touchpad_settings_changed(new_speed, accel, verbose);
                    inspector_terminal.emit(
                        "touchpad_settings",
                        &format!(
                            "{{\"mouse_mode\":{},\"speed\":{new_speed:.2},\"accel\":{accel},\"verbose_events\":{verbose}}}",
                            mouse_mode_terminal.load(std::sync::atomic::Ordering::SeqCst)
                        ),
                    );
                    println!("\n⚡ [驱动灵敏度] 鼠标速度已调整为: {new_speed:.2}x\n");
                } else {
                    println!("\n⚠️ 用法: s <速度数值>，例如: s 1.5\n");
                }
            } else if trimmed.eq_ignore_ascii_case("v") || trimmed.eq_ignore_ascii_case("verbose") {
                let (speed, accel, cur_verbose) = delegate_terminal.get_touchpad_settings();
                let new_verbose = !cur_verbose;
                delegate_terminal.on_touchpad_settings_changed(speed, accel, new_verbose);
                inspector_terminal.emit(
                    "touchpad_settings",
                    &format!(
                        "{{\"mouse_mode\":{},\"speed\":{speed:.2},\"accel\":{accel},\"verbose_events\":{new_verbose}}}",
                        mouse_mode_terminal.load(std::sync::atomic::Ordering::SeqCst)
                    ),
                );
                let desc = if new_verbose { "开启 (Info 级别)" } else { "静默 (Debug 级别)" };
                println!("\n📢 [事件日志输出] 已切换为: {desc}\n");
            }
        }
    });

    if let Some(port) = server.ui_port() {
        let url = format!("http://127.0.0.1:{port}");
        println!("Debug UI    : {url}");
        if open_browser {
            #[cfg(target_os = "macos")]
            let _ = tokio::process::Command::new("/usr/bin/open")
                .arg(&url)
                .spawn();
            #[cfg(not(target_os = "macos"))]
            let _ = tokio::process::Command::new("xdg-open")
                .arg(&url)
                .spawn();
        }
    }

    println!("atv-cli ready (press Ctrl+C to stop)");

    tokio::signal::ctrl_c().await.ok();
    println!("shutting down...");
    server.stop().await;
}
