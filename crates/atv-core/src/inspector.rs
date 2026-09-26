//! Real-time Inspector and Debug UI hub.
//!
//! Collects operations (buttons, touches, volume, auth/lifecycle state)
//! and broadcasts them over Server-Sent Events (SSE) to connected web browsers.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

const HISTORY_MAX_LEN: usize = 100;

pub struct InspectorHub {
    sequence: AtomicU64,
    history: Mutex<VecDeque<String>>,
    tx: broadcast::Sender<String>,
    client_connected: AtomicBool,
    client_peer: Mutex<Option<String>>,
    session_ready: AtomicBool,
}

impl InspectorHub {
    pub fn new() -> Arc<Self> {
        let (tx, _) = broadcast::channel(256);
        Arc::new(Self {
            sequence: AtomicU64::new(0),
            history: Mutex::new(VecDeque::with_capacity(HISTORY_MAX_LEN)),
            tx,
            client_connected: AtomicBool::new(false),
            client_peer: Mutex::new(None),
            session_ready: AtomicBool::new(false),
        })
    }

    pub fn set_client_connected(&self, peer: Option<&str>) {
        match peer {
            Some(p) => {
                self.client_connected.store(true, Ordering::SeqCst);
                if let Ok(mut g) = self.client_peer.lock() {
                    *g = Some(p.to_string());
                }
            }
            None => {
                self.client_connected.store(false, Ordering::SeqCst);
                if let Ok(mut g) = self.client_peer.lock() {
                    *g = None;
                }
                self.session_ready.store(false, Ordering::SeqCst);
            }
        }
    }

    pub fn set_session_ready(&self, ready: bool) {
        self.session_ready.store(ready, Ordering::SeqCst);
    }

    pub fn get_client_info(&self) -> (bool, Option<String>, bool) {
        let connected = self.client_connected.load(Ordering::SeqCst);
        let peer = self.client_peer.lock().ok().and_then(|g| g.clone());
        let ready = self.session_ready.load(Ordering::SeqCst);
        (connected, peer, ready)
    }

    /// Emit an event with JSON payload to all connected debug clients.
    pub fn emit(&self, event: &str, data_json: &str) {
        if self.tx.receiver_count() == 0 {
            return;
        }
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst) + 1;
        let now = chrono_timestamp();
        let packet = format!(
            "{{\"sequence\":{seq},\"timestamp\":\"{now}\",\"event\":\"{event}\",\"data\":{data_json}}}"
        );

        if let Ok(mut hist) = self.history.lock() {
            if hist.len() >= HISTORY_MAX_LEN {
                hist.pop_front();
            }
            hist.push_back(packet.clone());
        }

        let _ = self.tx.send(packet);
    }

    /// Subscribe to the live event stream, receiving recent history first.
    pub fn subscribe(&self) -> (Vec<String>, broadcast::Receiver<String>) {
        let hist = self
            .history
            .lock()
            .map(|h| h.iter().cloned().collect())
            .unwrap_or_default();
        let rx = self.tx.subscribe();
        (hist, rx)
    }
}

fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let millis = dur.subsec_millis();
    // Simple ISO 8601 formatting without extra external crate
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}.{:03}", hours, mins, s, millis)
}
