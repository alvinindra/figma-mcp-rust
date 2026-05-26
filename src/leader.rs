//! Axum-based leader: WebSocket bridge to the plugin plus HTTP /ping and /rpc
//! endpoints used by follower processes.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::bridge::Bridge;
use crate::schema::validate_rpc;
use crate::types::{RpcRequest, RpcResponse};

#[derive(Clone)]
struct AppState {
    bridge: Bridge,
    version: String,
}

pub struct Leader {
    pub bridge: Bridge,
    addr: SocketAddr,
    version: String,
    handle: Option<JoinHandle<()>>,
    cancel: CancellationToken,
}

impl Leader {
    pub fn new(addr: SocketAddr, version: String) -> Self {
        Self {
            bridge: Bridge::new(),
            addr,
            version,
            handle: None,
            cancel: CancellationToken::new(),
        }
    }

    /// Bind the port and serve. Returns immediately after the listener is up.
    pub async fn start(&mut self) -> std::io::Result<()> {
        let listener = TcpListener::bind(self.addr).await?;

        let state = AppState {
            bridge: self.bridge.clone(),
            version: self.version.clone(),
        };
        let app = Router::new()
            .route("/ping", get(handle_ping))
            .route("/rpc", post(handle_rpc))
            .route("/ws", get(handle_ws))
            .with_state(state);

        let cancel = self.cancel.clone();
        let addr = self.addr;
        let handle = tokio::spawn(async move {
            info!(target: "leader", "listening on {addr}");
            let server = axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(async move { cancel.cancelled().await });
            if let Err(e) = server.await {
                warn!(target: "leader", "serve error: {e}");
            }
        });
        self.handle = Some(handle);
        Ok(())
    }

    /// Stop the leader and close the bridge.
    pub async fn stop(&mut self) {
        self.cancel.cancel();
        if let Some(h) = self.handle.take() {
            let _ = h.await;
        }
        self.bridge.close().await;
    }
}

#[derive(Serialize)]
struct PingResponse {
    status: &'static str,
    version: String,
}

async fn handle_ping(State(state): State<AppState>) -> impl IntoResponse {
    Json(PingResponse {
        status: "ok",
        version: state.version.clone(),
    })
}

async fn handle_ws(
    ws: WebSocketUpgrade,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Cap inbound payloads at 100 MB to match the Go server's read-limit bump
    // for large Figma documents.
    let ws = ws
        .max_message_size(100 * 1024 * 1024)
        .max_frame_size(100 * 1024 * 1024);
    ws.on_upgrade(move |socket| async move {
        state.bridge.handle_socket(socket, remote.to_string()).await;
    })
}

async fn handle_rpc(
    State(state): State<AppState>,
    Json(req): Json<RpcRequest>,
) -> impl IntoResponse {
    info!(
        target: "leader",
        "rpc {} nodeIDs={:?}",
        req.tool, req.node_ids
    );

    if let Some(err) = validate_rpc(&req.tool, &req.node_ids, &req.params) {
        warn!(target: "leader", "rpc {} validation error: {}", req.tool, err);
        return (
            StatusCode::BAD_REQUEST,
            Json(RpcResponse {
                data: None,
                error: err,
            }),
        );
    }

    match state
        .bridge
        .send(&req.tool, req.node_ids.clone(), req.params)
        .await
    {
        Ok(resp) => {
            if !resp.error.is_empty() {
                warn!(target: "leader", "rpc {} plugin error: {}", req.tool, resp.error);
                (
                    StatusCode::OK,
                    Json(RpcResponse {
                        data: None,
                        error: resp.error,
                    }),
                )
            } else {
                (
                    StatusCode::OK,
                    Json(RpcResponse {
                        data: resp.data,
                        error: String::new(),
                    }),
                )
            }
        }
        Err(e) => {
            warn!(target: "leader", "rpc {} bridge error: {}", req.tool, e);
            (
                StatusCode::OK,
                Json(RpcResponse {
                    data: None,
                    error: e.to_string(),
                }),
            )
        }
    }
}

/// Helper for callers that want raw shared `Arc<Bridge>`. (Currently unused but useful for tests.)
pub fn shared(b: &Bridge) -> Arc<Bridge> {
    Arc::new(b.clone())
}
