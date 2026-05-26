//! HTTP client that proxies MCP tool calls to the active leader.

use std::time::Duration;

use serde_json::Map;
use tracing::{debug, warn};

use crate::error::BridgeError;
use crate::types::{BridgeResponse, RpcRequest, RpcResponse};

#[derive(Clone)]
pub struct Follower {
    leader_url: String,
    client: reqwest::Client,
}

impl Follower {
    pub fn new(leader_url: impl Into<String>) -> Self {
        // 35s > 30s bridge timeout — gives the leader time to time out first.
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(35))
            .build()
            .expect("reqwest client build");
        Self {
            leader_url: leader_url.into(),
            client,
        }
    }

    pub async fn send(
        &self,
        tool: &str,
        node_ids: Vec<String>,
        params: Map<String, serde_json::Value>,
    ) -> Result<BridgeResponse, BridgeError> {
        debug!(
            target: "follower",
            "proxy {} nodeIDs={:?} params={:?} → {}/rpc",
            tool, node_ids, params, self.leader_url
        );
        let start = std::time::Instant::now();

        let req = RpcRequest {
            tool: tool.into(),
            node_ids,
            params,
        };
        let resp = self
            .client
            .post(format!("{}/rpc", self.leader_url))
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                warn!(target: "follower", "proxy {} rpc error: {}", tool, e);
                BridgeError::Send(format!("rpc call: {e}"))
            })?;

        let rpc: RpcResponse = resp
            .json()
            .await
            .map_err(|e| BridgeError::Send(e.to_string()))?;
        if !rpc.error.is_empty() {
            warn!(
                target: "follower",
                "proxy {} error from leader in {}ms: {}",
                tool,
                start.elapsed().as_millis(),
                rpc.error
            );
            return Ok(BridgeResponse {
                error: rpc.error,
                ..Default::default()
            });
        }
        debug!(
            target: "follower",
            "proxy {} ok in {}ms",
            tool,
            start.elapsed().as_millis()
        );
        Ok(BridgeResponse {
            r#type: tool.into(),
            data: rpc.data,
            ..Default::default()
        })
    }

    /// Health check.
    pub async fn ping(&self) -> bool {
        let url = format!("{}/ping", self.leader_url);
        let resp = match self
            .client
            .get(&url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                debug!(target: "follower", "ping {} failed: {}", self.leader_url, e);
                return false;
            }
        };
        let ok = resp.status().is_success();
        debug!(target: "follower", "ping {} → {} (healthy={})", self.leader_url, resp.status(), ok);
        ok
    }
}
