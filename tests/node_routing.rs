//! End-to-end smoke test: a follower forwards via HTTP to a stubbed leader.
//!
//! This exercises the leader's `/rpc` route and the follower's HTTP client without
//! involving the WebSocket bridge or a real Figma plugin.

use axum::{routing::post, Json, Router};
use figma_mcp_rust::follower::Follower;
use figma_mcp_rust::types::{RpcRequest, RpcResponse};
use serde_json::{json, Map};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;

#[tokio::test]
async fn follower_proxies_rpc_to_leader() {
    // Stub leader: echoes the tool name in `data.echoedTool`.
    async fn rpc(Json(req): Json<RpcRequest>) -> Json<RpcResponse> {
        Json(RpcResponse {
            data: Some(json!({ "echoedTool": req.tool, "nodeIds": req.node_ids })),
            error: String::new(),
        })
    }

    let app = Router::new().route("/rpc", post(rpc));
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
    let listener = TcpListener::bind(addr).await.unwrap();
    let bound = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    let follower = Follower::new(format!("http://{}", bound));
    let resp = follower
        .send("get_metadata", vec!["1:1".into()], Map::new())
        .await
        .expect("send");
    let data = resp.data.expect("data");
    assert_eq!(data["echoedTool"], "get_metadata");
    assert_eq!(data["nodeIds"][0], "1:1");
}

#[tokio::test]
async fn follower_returns_leader_error_inline() {
    async fn rpc(Json(_req): Json<RpcRequest>) -> Json<RpcResponse> {
        Json(RpcResponse {
            data: None,
            error: "plugin not connected".into(),
        })
    }

    let app = Router::new().route("/rpc", post(rpc));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let bound = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    let follower = Follower::new(format!("http://{}", bound));
    let resp = follower
        .send("get_metadata", vec![], Map::new())
        .await
        .expect("send");
    assert_eq!(resp.error, "plugin not connected");
    assert!(resp.data.is_none());
}
