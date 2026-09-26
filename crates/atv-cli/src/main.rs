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
           --name <NAME>                  Device name shown in iOS Remote (default: \"Mac Remote\")\n\
           --pin <PIN>                    4-digit pairing PIN (default: 1111)\n\
           --ip <IPV4>                    Local IPv4 to advertise (auto-detected if omitted)\n\
           --mouse                        Trackpad mouse mode (touch moves cursor, tap clicks)\n\
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
            "--mouse" => config.mouse_mode = true,
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
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "atv_core=info,atv_cli=info".into()),
        )
        .init();

    let args = parse_args();
    let mouse_mode = args.config.mouse_mode;
    let open_browser = args.open_browser;

    let delegate: Arc<dyn AtvDelegate> = match args.target {
        Target::Mac => {
            #[cfg(target_os = "macos")]
            {
                println!("Target    : macOS (Mac mini control via CoreGraphics)");
                Arc::new(macos::MacDelegate::new(mouse_mode))
            }
            #[cfg(not(target_os = "macos"))]
            {
                eprintln!("Warning: Target 'mac' is only supported on macOS; falling back to print");
                Arc::new(print::PrintDelegate)
            }
        }
        Target::Android => {
            let host = args.android_host.expect("checked above");
            println!("Target    : Android TV at {host}:{}", args.android_port);
            Arc::new(android_adb::AndroidAdbDelegate::new(
                host,
                args.android_port,
                mouse_mode,
            ))
        }
        Target::Print => {
            println!("Target    : observe-only (printing events to stdout)");
            Arc::new(print::PrintDelegate)
        }
    };

    let server = match AtvServer::start(args.config, delegate).await {
        Ok(server) => server,
        Err(e) => {
            eprintln!("failed to start server: {e}");
            exit(1);
        }
    };

    if let Some(port) = server.ui_port() {
        let url = format!("http://127.0.0.1:{port}");
        println!("Debug UI  : {url}");
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
