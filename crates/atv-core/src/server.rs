//! Server: TCP listeners and Bonjour (mDNS) advertisement.
//!
//! Ported from `fake_atv.py`'s `main()`: three services with the exact TXT
//! records iOS expects (`rpFl=0x36782` etc. gate features like the volume
//! keys), the MRP port answering minimal framing, and the AirPlay port
//! answering HTTP "OK".

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

#[cfg(not(target_os = "macos"))]
use mdns_sd::{ServiceDaemon, ServiceInfo};
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::delegate::{AtvDelegate, EventKind};
use crate::error::{Error, Result};
use crate::inspector::InspectorHub;
use crate::session::{CompanionSession, DEVICE_MODEL, SERVER_IDENTIFIER};
use crate::FrameType;

pub const MRP_PORT: u16 = 49152;
pub const COMPANION_PORT: u16 = 49153;
pub const SOURCE_VERSION: &str = "715.2";
pub const DEFAULT_DEVICE_NAME: &str = "Mac Remote";

/// Server configuration.
#[derive(Debug, Clone)]
pub struct AtvConfig {
    /// Device name shown in Apple TV Remote.
    pub name: String,
    /// Four-digit pairing PIN (0-9999).
    pub pin: u32,
    /// LAN IPv4 address to advertise; auto-detected when None.
    pub ip: Option<Ipv4Addr>,
    /// Trackpad mode: touch drives pointer deltas instead of direction keys.
    pub mouse_mode: bool,
    /// Port for Debug Web UI & Inspector (None to disable, default Some(8765)).
    pub ui_port: Option<u16>,
}

impl Default for AtvConfig {
    fn default() -> Self {
        Self {
            name: DEFAULT_DEVICE_NAME.to_string(),
            pin: 1111,
            ip: None,
            mouse_mode: false,
            ui_port: Some(8765),
        }
    }
}

/// A running Apple TV simulator. Drop or call `stop()` to shut down.
pub struct AtvServer {
    config: AtvConfig,
    ip: Ipv4Addr,
    airplay_port: u16,
    ui_port: Option<u16>,
    inspector: Arc<InspectorHub>,
    shutdown_tx: watch::Sender<bool>,
    tasks: Vec<JoinHandle<()>>,
    #[cfg(target_os = "macos")]
    dns_sd_children: Vec<tokio::process::Child>,
    #[cfg(not(target_os = "macos"))]
    mdns: ServiceDaemon,
    #[cfg(not(target_os = "macos"))]
    mdns_fullnames: Vec<String>,
}

impl AtvServer {
    /// Start listeners and advertise the three Bonjour services.
    pub async fn start(config: AtvConfig, delegate: Arc<dyn AtvDelegate>) -> Result<Self> {
        if config.pin > 9999 {
            return Err(Error::Protocol("pin must be between 0000 and 9999".into()));
        }
        let ip = match config.ip {
            Some(ip) => ip,
            None => detect_local_ip()?,
        };

        let mrp_listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, MRP_PORT)).await?;
        let companion_listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, COMPANION_PORT)).await?;
        // Port 7000 is normally occupied when macOS AirPlay Receiver is on;
        // we only need this endpoint for discovery, so let the OS choose.
        let airplay_listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
        let airplay_port = airplay_listener.local_addr()?.port();

        let inspector = InspectorHub::new();

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let mut tasks = Vec::new();

        tasks.push(tokio::spawn(mrp_loop(mrp_listener, shutdown_rx.clone())));
        tasks.push(tokio::spawn(airplay_loop(
            airplay_listener,
            shutdown_rx.clone(),
        )));
        tasks.push(tokio::spawn(companion_loop(
            companion_listener,
            config.clone(),
            delegate.clone(),
            inspector.clone(),
            shutdown_rx.clone(),
        )));

        let mut actual_ui_port = None;
        if let Some(ui_p) = config.ui_port {
            match TcpListener::bind((Ipv4Addr::UNSPECIFIED, ui_p)).await {
                Ok(ui_listener) => {
                    actual_ui_port = Some(ui_p);
                    tasks.push(tokio::spawn(ui_loop(
                        ui_listener,
                        inspector.clone(),
                        delegate.clone(),
                        shutdown_rx.clone(),
                    )));
                }
                Err(e) => {
                    warn!("failed to bind Debug UI on port {ui_p}: {e}");
                }
            }
        }

        let unique_id = SERVER_IDENTIFIER.replace('-', "");
        let device_id = "AA:BB:CC:DD:EE:01";

        let services: Vec<(&str, u16, Vec<(&str, &str)>)> = vec![
            (
                "_mediaremotetv._tcp.local.",
                MRP_PORT,
                vec![
                    ("Name", config.name.as_str()),
                    ("UniqueIdentifier", unique_id.as_str()),
                    ("SystemBuildVersion", "22K160"),
                    ("LocalAirPlayReceiverPairingIdentity", "9C4F2B8A1D3E5F60"),
                    ("ModelName", "Apple TV"),
                    ("AllowPairing", "YES"),
                ],
            ),
            (
                "_companion-link._tcp.local.",
                COMPANION_PORT,
                vec![
                    ("rpMac", "1"),
                    ("rpHA", "D851F0A4E5C9"),
                    ("rpHN", "B7359A9BCBAB"),
                    ("rpVr", SOURCE_VERSION),
                    ("rpMd", DEVICE_MODEL),
                    ("rpFl", "0x36782"),
                    ("rpAD", "F0E18C86DB60"),
                    ("rpHI", "40DB206B32FA"),
                    ("rpBA", device_id),
                ],
            ),
            (
                "_airplay._tcp.local.",
                airplay_port,
                vec![
                    ("deviceid", device_id),
                    ("features", "0x5A7FFFF7,0x1E"),
                    ("flags", "0x44"),
                    ("model", DEVICE_MODEL),
                    ("srcvers", SOURCE_VERSION),
                    ("vv", "2"),
                    ("pi", SERVER_IDENTIFIER),
                    ("pk", "6b8b4567f85b7f54a3e1c0a93f0a9e2c"),
                    ("name", config.name.as_str()),
                ],
            ),
        ];

        #[cfg(target_os = "macos")]
        let mut dns_sd_children = Vec::new();
        #[cfg(target_os = "macos")]
        {
            for (service_type, port, properties) in &services {
                let dns_sd_type = service_type
                    .trim_end_matches(".local.")
                    .trim_end_matches(".local");
                let mut cmd = tokio::process::Command::new("/usr/bin/dns-sd");
                cmd.arg("-R")
                    .arg(&config.name)
                    .arg(dns_sd_type)
                    .arg("local.")
                    .arg(port.to_string());
                for (k, v) in properties {
                    cmd.arg(format!("{k}={v}"));
                }
                cmd.stdout(std::process::Stdio::null());
                cmd.stderr(std::process::Stdio::null());
                match cmd.spawn() {
                    Ok(child) => {
                        dns_sd_children.push(child);
                        info!("registered {}.{service_type} on {ip}:{port} (via macOS mDNSResponder)", config.name);
                    }
                    Err(e) => {
                        warn!("failed to spawn /usr/bin/dns-sd for {service_type}: {e}");
                    }
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        #[cfg(not(target_os = "macos"))]
        let mdns = ServiceDaemon::new().map_err(|e| Error::Mdns(e.to_string()))?;
        #[cfg(not(target_os = "macos"))]
        let mut mdns_fullnames = Vec::new();
        #[cfg(not(target_os = "macos"))]
        {
            let host_name = format!("{}.local.", config.name.replace(' ', "-"));
            for (service_type, port, properties) in services {
                let info = ServiceInfo::new(
                    service_type,
                    &config.name,
                    &host_name,
                    ip.to_string(),
                    port,
                    &properties[..],
                )
                .map_err(|e| Error::Mdns(e.to_string()))?;
                mdns_fullnames.push(info.get_fullname().to_string());
                mdns.register(info).map_err(|e| Error::Mdns(e.to_string()))?;
                info!("registered {}.{service_type} on {ip}:{port}", config.name);
            }
        }

        info!("fake Apple TV started");
        info!("PIN       : {:04}", config.pin);
        info!("MRP       : {ip}:{MRP_PORT}");
        info!("Companion : {ip}:{COMPANION_PORT}");
        info!("AirPlay   : {ip}:{airplay_port}");
        if let Some(port) = actual_ui_port {
            info!("Debug UI  : http://127.0.0.1:{port}");
        }

        Ok(Self {
            config,
            ip,
            airplay_port,
            ui_port: actual_ui_port,
            inspector,
            shutdown_tx,
            tasks,
            #[cfg(target_os = "macos")]
            dns_sd_children,
            #[cfg(not(target_os = "macos"))]
            mdns,
            #[cfg(not(target_os = "macos"))]
            mdns_fullnames,
        })
    }

    pub fn config(&self) -> &AtvConfig {
        &self.config
    }

    /// The LAN address the services advertise.
    pub fn ip(&self) -> Ipv4Addr {
        self.ip
    }

    pub fn mrp_port(&self) -> u16 {
        MRP_PORT
    }

    pub fn companion_port(&self) -> u16 {
        COMPANION_PORT
    }

    pub fn airplay_port(&self) -> u16 {
        self.airplay_port
    }

    pub fn ui_port(&self) -> Option<u16> {
        self.ui_port
    }

    pub fn inspector(&self) -> Arc<InspectorHub> {
        self.inspector.clone()
    }

    /// Unregister Bonjour services and stop all listeners.
    pub async fn stop(self) {
        let _ = self.shutdown_tx.send(true);

        #[cfg(target_os = "macos")]
        for mut child in self.dns_sd_children {
            let _ = child.kill().await;
        }

        #[cfg(not(target_os = "macos"))]
        {
            for fullname in &self.mdns_fullnames {
                if let Err(e) = self.mdns.unregister(fullname) {
                    warn!("mDNS unregister failed for {fullname}: {e}");
                }
            }
            if let Err(e) = self.mdns.shutdown() {
                warn!("mDNS shutdown failed: {e}");
            }
        }

        for task in &self.tasks {
            task.abort();
        }
        for task in self.tasks {
            let _ = task.await;
        }
    }
}

/// Return the IPv4 address used to reach the local network (UDP-connect
/// trick, like `fake_atv.detect_local_ip`).
pub fn detect_local_ip() -> Result<Ipv4Addr> {
    let sock = std::net::UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
    // A UDP connect sends nothing; it asks the kernel which route it would use.
    sock.connect((Ipv4Addr::new(192, 0, 2, 1), 9))?;
    match sock.local_addr()?.ip() {
        std::net::IpAddr::V4(ip) => Ok(ip),
        _ => Ok(Ipv4Addr::LOCALHOST),
    }
}

async fn companion_loop(
    listener: TcpListener,
    config: AtvConfig,
    delegate: Arc<dyn AtvDelegate>,
    inspector: Arc<InspectorHub>,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        let stream = tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => stream,
                Err(e) => {
                    warn!("companion accept failed: {e}");
                    continue;
                }
            },
            _ = shutdown.changed() => break,
        };
        let config = config.clone();
        let delegate = delegate.clone();
        let inspector = inspector.clone();
        tokio::spawn(handle_companion(stream, config, delegate, inspector));
    }
}

async fn handle_companion(
    stream: TcpStream,
    config: AtvConfig,
    delegate: Arc<dyn AtvDelegate>,
    inspector: Arc<InspectorHub>,
) {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "?".into());
    info!("companion client connected: {peer}");
    delegate.on_event(EventKind::ClientConnected, &peer);
    inspector.emit("client_connected", &format!("{{\"peer\":\"{peer}\"}}"));

    let (mut reader, writer) = stream.into_split();
    let writer = Arc::new(Mutex::new(writer));
    let mut session = CompanionSession::new(
        writer,
        delegate.clone(),
        config.name.clone(),
        config.pin,
        config.mouse_mode,
        inspector.clone(),
    );
    let mut disconnect_reason = "peer_closed".to_string();

    loop {
        let mut header = [0u8; 4];
        match reader.read_exact(&mut header).await {
            Ok(_) => {}
            Err(e) => {
                if e.kind() != std::io::ErrorKind::UnexpectedEof {
                    disconnect_reason = format!("error:{e}");
                }
                break;
            }
        }
        let Some(frame_type) = FrameType::from_u8(header[0]) else {
            disconnect_reason = format!("error:unknown frame type {}", header[0]);
            warn!("unknown frame type {}", header[0]);
            break;
        };
        let length = u32::from_be_bytes([0, header[1], header[2], header[3]]) as usize;
        let mut payload = vec![0u8; length];
        if let Err(e) = reader.read_exact(&mut payload).await {
            disconnect_reason = format!("incomplete_frame:{e}");
            break;
        }
        info!("companion frame={:?} length={length}", frame_type);
        if let Err(e) = session.handle_frame(frame_type, &payload, &header).await {
            disconnect_reason = format!("error:{e}");
            warn!("companion error: {e}");
            break;
        }
    }

    info!("companion client disconnected: {peer}");
    delegate.on_event(
        EventKind::ClientDisconnected,
        &format!("{peer} reason={disconnect_reason}"),
    );
    inspector.emit(
        "client_disconnected",
        &format!("{{\"peer\":\"{peer}\",\"reason\":\"{disconnect_reason}\"}}"),
    );
}

const DASHBOARD_HTML: &str = include_str!("dashboard.html");

async fn ui_loop(
    listener: TcpListener,
    inspector: Arc<InspectorHub>,
    delegate: Arc<dyn AtvDelegate>,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        let stream = tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => stream,
                Err(e) => {
                    warn!("UI accept failed: {e}");
                    continue;
                }
            },
            _ = shutdown.changed() => break,
        };
        let inspector = inspector.clone();
        let delegate = delegate.clone();
        tokio::spawn(handle_ui_client(stream, inspector, delegate));
    }
}

async fn handle_ui_client(
    mut stream: TcpStream,
    inspector: Arc<InspectorHub>,
    delegate: Arc<dyn AtvDelegate>,
) {
    use tokio::io::AsyncWriteExt;

    let mut buf = vec![0u8; 4096];
    let n = match stream.read(&mut buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    let req = String::from_utf8_lossy(&buf[..n]);
    let first_line = req.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0];
    let path = parts[1];

    match (method, path) {
        ("GET", "/") | ("GET", "/index.html") => {
            let body = DASHBOARD_HTML.as_bytes();
            let header = format!(
                "HTTP/1.1 200 OK\r\n\
                 Content-Type: text/html; charset=utf-8\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\
                 \r\n",
                body.len()
            );
            let _ = stream.write_all(header.as_bytes()).await;
            let _ = stream.write_all(body).await;
            let _ = stream.flush().await;
        }
        ("GET", "/events") => {
            let header = "HTTP/1.1 200 OK\r\n\
                          Content-Type: text/event-stream\r\n\
                          Cache-Control: no-cache\r\n\
                          Connection: keep-alive\r\n\
                          Access-Control-Allow-Origin: *\r\n\
                          \r\n";
            if stream.write_all(header.as_bytes()).await.is_err() {
                return;
            }
            if stream.flush().await.is_err() {
                return;
            }

            let (history, mut rx) = inspector.subscribe();
            for item in history {
                let msg = format!("data: {item}\n\n");
                if stream.write_all(msg.as_bytes()).await.is_err() {
                    return;
                }
            }
            let _ = stream.flush().await;

            let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(15));
            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Ok(packet) => {
                                let sse = format!("data: {packet}\n\n");
                                if stream.write_all(sse.as_bytes()).await.is_err() {
                                    break;
                                }
                                if stream.flush().await.is_err() {
                                    break;
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                            Err(_) => break,
                        }
                    }
                    _ = ping_interval.tick() => {
                        if stream.write_all(b": ping\n\n").await.is_err() {
                            break;
                        }
                        if stream.flush().await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
        ("POST", "/api/action") => {
            if let Some(pos) = req.find("\r\n\r\n") {
                let body = &req[pos + 4..];
                if let Some(button) = extract_button_from_json(body) {
                    delegate.on_button(&button);
                    inspector.emit("button", &format!("{{\"name\":\"{button}\"}}"));
                    let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 15\r\n\r\n{\"status\":\"ok\"}";
                    let _ = stream.write_all(resp.as_bytes()).await;
                    return;
                }
            }
            let resp = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes()).await;
        }
        ("GET", "/favicon.ico") => {
            let resp = "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes()).await;
        }
        _ => {
            let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes()).await;
        }
    }
}

fn extract_button_from_json(json: &str) -> Option<String> {
    let key = "\"button\"";
    let idx = json.find(key)?;
    let rest = &json[idx + key.len()..];
    let colon = rest.find(':')?;
    let rest = rest[colon + 1..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    let rest = &rest[1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

async fn mrp_loop(listener: TcpListener, mut shutdown: watch::Receiver<bool>) {
    loop {
        let stream = tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => stream,
                Err(e) => {
                    warn!("MRP accept failed: {e}");
                    continue;
                }
            },
            _ = shutdown.changed() => break,
        };
        tokio::spawn(handle_mrp(stream));
    }
}

async fn handle_mrp(mut stream: TcpStream) {
    let peer = peer_string(&stream);
    info!("MRP client connected: {peer}");
    loop {
        let Some(size) = read_varint(&mut stream).await else {
            break;
        };
        let mut body = vec![0u8; size];
        if stream.read_exact(&mut body).await.is_err() {
            break;
        }
        debug!("MRP message size={size}");
    }
    info!("MRP client disconnected: {peer}");
}

async fn read_varint(stream: &mut TcpStream) -> Option<usize> {
    let mut shift = 0u32;
    let mut result: usize = 0;
    loop {
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte).await.ok()?;
        result |= ((byte[0] & 0x7F) as usize) << shift;
        if byte[0] & 0x80 == 0 {
            return Some(result);
        }
        shift += 7;
        if shift > 63 {
            warn!("varint too long");
            return None;
        }
    }
}

async fn airplay_loop(listener: TcpListener, mut shutdown: watch::Receiver<bool>) {
    loop {
        let stream = tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => stream,
                Err(e) => {
                    warn!("AirPlay accept failed: {e}");
                    continue;
                }
            },
            _ = shutdown.changed() => break,
        };
        tokio::spawn(handle_airplay(stream));
    }
}

async fn handle_airplay(mut stream: TcpStream) {
    let peer = peer_string(&stream);
    info!("AirPlay client connected: {peer}");
    let mut buf = [0u8; 4096];
    let _ = stream.read(&mut buf).await;
    use tokio::io::AsyncWriteExt;
    let _ = stream
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nOK")
        .await;
}

fn peer_string(stream: &TcpStream) -> String {
    stream
        .peer_addr()
        .map(|a: SocketAddr| a.to_string())
        .unwrap_or_else(|_| "?".into())
}
