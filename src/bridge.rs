//! WebSocket bridge to the Figma plugin with request/response correlation.
//!
//! Improvements over the Go original:
//! - Uses `tokio::sync::oneshot` per pending request, which eliminates the
//!   "send-on-closed-channel" race the Go version guards against with `sync.Once`.
//! - Timeout is enforced by `tokio::time::timeout` inside `send`, so no separate
//!   timer goroutine or `AfterFunc` is needed.
//! - Progress frames extend the deadline using `tokio::sync::Notify` per request.
//! - Single connection at a time, like the original.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures::stream::StreamExt;
use futures::SinkExt;
use serde_json::Map;
use tokio::sync::{oneshot, Mutex, Notify, RwLock};
use tokio::time::timeout_at;
use tracing::{debug, info, warn};

use crate::error::BridgeError;
use crate::types::{BridgeRequest, BridgeResponse};

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const GET_DOCUMENT_TIMEOUT_SECS: u64 = 60;
const PROGRESS_EXTENSION_SECS: u64 = 60;

/// One in-flight request.
struct Pending {
    /// Resolved when the plugin replies (or the request is aborted).
    sender: oneshot::Sender<BridgeResponse>,
    /// Notified each time a progress frame arrives so `send()` can extend its deadline.
    progress: Arc<Notify>,
}

type WsSink = futures::stream::SplitSink<WebSocket, Message>;

/// State of the active WebSocket connection (if any) and the pending requests map.
struct Inner {
    /// Active WebSocket sink. `None` when no plugin is connected.
    sink: RwLock<Option<Arc<Mutex<WsSink>>>>,
    pending: DashMap<String, Pending>,
    counter: AtomicI64,
}

/// Bridge between MCP server and Figma plugin.
#[derive(Clone)]
pub struct Bridge {
    inner: Arc<Inner>,
}

impl Bridge {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                sink: RwLock::new(None),
                pending: DashMap::new(),
                counter: AtomicI64::new(0),
            }),
        }
    }

    pub async fn is_connected(&self) -> bool {
        self.inner.sink.read().await.is_some()
    }

    /// Accept an upgraded WebSocket and run its read loop.
    /// If an existing connection is present, it is closed (latest-wins, same as Go).
    pub async fn handle_socket(&self, socket: WebSocket, remote: String) {
        let (sink, mut stream) = socket.split();
        let sink = Arc::new(Mutex::new(sink));

        let replaced = {
            let mut slot = self.inner.sink.write().await;
            let prev = slot.replace(sink.clone());
            prev.is_some()
        };

        if replaced {
            info!(target: "bridge", "plugin connected (replaced previous connection) from {remote}");
        } else {
            info!(target: "bridge", "plugin connected from {remote}");
        }

        // Read loop
        while let Some(msg) = stream.next().await {
            let msg = match msg {
                Ok(m) => m,
                Err(e) => {
                    warn!(target: "bridge", "read error: {e}");
                    break;
                }
            };
            match msg {
                Message::Text(text) => self.dispatch_text(&text),
                Message::Binary(bin) => match std::str::from_utf8(&bin) {
                    Ok(text) => self.dispatch_text(text),
                    Err(_) => warn!(target: "bridge", "received non-utf8 binary frame"),
                },
                Message::Ping(_) | Message::Pong(_) => {}
                Message::Close(_) => break,
            }
        }

        // Clear the active sink only if it's still ours (a newer connection may have replaced us).
        {
            let mut slot = self.inner.sink.write().await;
            if let Some(active) = slot.as_ref() {
                if Arc::ptr_eq(active, &sink) {
                    *slot = None;
                }
            }
        }
        info!(target: "bridge", "plugin disconnected");
    }

    fn dispatch_text(&self, text: &str) {
        let resp: BridgeResponse = match serde_json::from_str(text) {
            Ok(r) => r,
            Err(e) => {
                warn!(target: "bridge", "decode error: {e}");
                return;
            }
        };

        // Progress frames: just bump the deadline.
        if resp.progress > 0 && !resp.request_id.is_empty() {
            if let Some(entry) = self.inner.pending.get(&resp.request_id) {
                entry.progress.notify_one();
                debug!(
                    target: "bridge",
                    "progress {}: {}% {}", resp.request_id, resp.progress, resp.message
                );
            } else {
                debug!(
                    target: "bridge",
                    "progress {} no pending entry (already resolved or timed out)", resp.request_id
                );
            }
            return;
        }

        if resp.request_id.is_empty() {
            warn!(target: "bridge", "received message with empty requestId — ignored");
            return;
        }

        if let Some((_, pending)) = self.inner.pending.remove(&resp.request_id) {
            if !resp.error.is_empty() {
                debug!(target: "bridge", "← {} error: {}", resp.request_id, resp.error);
            } else {
                debug!(target: "bridge", "← {} ok", resp.request_id);
            }
            // Receiver may have been dropped if send() already gave up; ignore the error.
            let _ = pending.sender.send(resp);
        } else {
            debug!(
                target: "bridge",
                "← {} received but no pending entry (timed out?)", resp.request_id
            );
        }
    }

    /// Send a request to the plugin and wait for the response.
    pub async fn send(
        &self,
        request_type: &str,
        node_ids: Vec<String>,
        params: Map<String, serde_json::Value>,
    ) -> Result<BridgeResponse, BridgeError> {
        let sink = {
            let slot = self.inner.sink.read().await;
            match slot.as_ref() {
                Some(s) => s.clone(),
                None => return Err(BridgeError::NotConnected),
            }
        };

        let request_id = self.next_id();
        let req = BridgeRequest {
            r#type: request_type.into(),
            request_id: request_id.clone(),
            node_ids,
            params,
        };

        let (tx, rx) = oneshot::channel();
        let progress = Arc::new(Notify::new());
        self.inner.pending.insert(
            request_id.clone(),
            Pending {
                sender: tx,
                progress: progress.clone(),
            },
        );

        debug!(
            target: "bridge",
            "→ {} {} nodeIDs={:?} params={:?}",
            request_id, request_type, req.node_ids, req.params
        );
        let start = Instant::now();

        let payload = serde_json::to_string(&req)?;
        let write_result = {
            let mut guard = sink.lock().await;
            guard.send(Message::Text(payload)).await
        };
        if let Err(e) = write_result {
            self.inner.pending.remove(&request_id);
            warn!(target: "bridge", "→ {} write error: {}", request_id, e);
            return Err(BridgeError::Send(e.to_string()));
        }

        // Wait, extending the deadline whenever a progress frame arrives.
        let base = if request_type == "get_document" {
            GET_DOCUMENT_TIMEOUT_SECS
        } else {
            DEFAULT_TIMEOUT_SECS
        };
        let result = wait_with_progress(rx, progress, base).await;

        match result {
            Ok(resp) => {
                debug!(
                    target: "bridge",
                    "→ {} {} completed in {}ms",
                    request_id,
                    request_type,
                    start.elapsed().as_millis()
                );
                Ok(resp)
            }
            Err(e) => {
                self.inner.pending.remove(&request_id);
                if matches!(e, BridgeError::Timeout) {
                    warn!(
                        target: "bridge",
                        "→ {} {} timed out after {}s", request_id, request_type, base
                    );
                }
                Err(e)
            }
        }
    }

    /// Close the connection, rejecting all pending requests.
    pub async fn close(&self) {
        // Drop pending entries — receivers will see channel-closed → Cancelled.
        self.inner.pending.clear();
        let mut slot = self.inner.sink.write().await;
        if let Some(sink) = slot.take() {
            let mut guard = sink.lock().await;
            let _ = guard.send(Message::Close(None)).await;
        }
    }

    fn next_id(&self) -> String {
        let n = self.inner.counter.fetch_add(1, Ordering::SeqCst) + 1;
        // Approximate the Go format "req-HHMMSS-N" using local wall clock seconds.
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let hh = (secs / 3600) % 24;
        let mm = (secs / 60) % 60;
        let ss = secs % 60;
        format!("req-{:02}{:02}{:02}-{}", hh, mm, ss, n)
    }
}

async fn wait_with_progress(
    mut rx: oneshot::Receiver<BridgeResponse>,
    progress: Arc<Notify>,
    base_secs: u64,
) -> Result<BridgeResponse, BridgeError> {
    let mut deadline = tokio::time::Instant::now() + Duration::from_secs(base_secs);
    loop {
        tokio::select! {
            biased;
            res = &mut rx => {
                return res.map_err(|_| BridgeError::Cancelled);
            }
            _ = progress.notified() => {
                deadline = tokio::time::Instant::now() + Duration::from_secs(PROGRESS_EXTENSION_SECS);
            }
            r = timeout_at(deadline, std::future::pending::<()>()) => {
                if r.is_err() {
                    return Err(BridgeError::Timeout);
                }
            }
        }
    }
}

impl Default for Bridge {
    fn default() -> Self {
        Self::new()
    }
}
