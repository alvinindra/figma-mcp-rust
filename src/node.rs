//! Node — routes MCP tool calls to either the leader's bridge or the follower's
//! HTTP proxy depending on the current role.

use std::net::SocketAddr;
use std::sync::Arc;

use serde_json::{Map, Value};
use tokio::sync::RwLock;
use tracing::info;

use crate::error::BridgeError;
use crate::follower::Follower;
use crate::leader::Leader;
use crate::schema::normalize_node_id;
use crate::types::{BridgeResponse, Role};

pub struct Node {
    addr: SocketAddr,
    version: String,
    state: RwLock<NodeState>,
}

struct NodeState {
    role: Role,
    leader: Option<Leader>,
    follower: Follower,
}

impl Node {
    pub fn new(addr: SocketAddr, version: String) -> Self {
        let follower = Follower::new(format!("http://{}", addr));
        Self {
            addr,
            version,
            state: RwLock::new(NodeState {
                role: Role::Unknown,
                leader: None,
                follower,
            }),
        }
    }

    pub async fn role(&self) -> Role {
        self.state.read().await.role
    }

    pub async fn role_name(&self) -> &'static str {
        self.role().await.as_str()
    }

    /// Send a tool call to whichever backend matches the current role.
    pub async fn send(
        self: &Arc<Self>,
        tool: &str,
        mut node_ids: Vec<String>,
        mut params: Map<String, Value>,
    ) -> Result<BridgeResponse, BridgeError> {
        for id in node_ids.iter_mut() {
            *id = normalize_node_id(id);
        }
        for key in ["nodeId", "parentId"] {
            if let Some(Value::String(s)) = params.get_mut(key) {
                *s = normalize_node_id(s);
            }
        }

        let (role, leader_bridge) = {
            let state = self.state.read().await;
            let bridge = state.leader.as_ref().map(|l| l.bridge.clone());
            (state.role, bridge)
        };

        info!(target: "node", "tool={} role={} nodeIDs={:?}", tool, role.as_str(), node_ids);

        if role == Role::Leader {
            if let Some(bridge) = leader_bridge {
                return bridge.send(tool, node_ids, params).await;
            }
        }
        let follower = self.state.read().await.follower.clone();
        follower.send(tool, node_ids, params).await
    }

    /// Attempt to bind the port and become the leader. Returns an error if the port is taken.
    pub async fn become_leader(self: &Arc<Self>) -> std::io::Result<()> {
        let mut state = self.state.write().await;
        if state.role == Role::Leader {
            return Ok(());
        }
        let mut leader = Leader::new(self.addr, self.version.clone());
        leader.start().await?;
        state.leader = Some(leader);
        state.role = Role::Leader;
        info!(target: "node", "became LEADER");
        Ok(())
    }

    pub async fn become_follower(self: &Arc<Self>) {
        let mut state = self.state.write().await;
        if state.role == Role::Follower {
            return;
        }
        if let Some(mut l) = state.leader.take() {
            l.stop().await;
        }
        state.role = Role::Follower;
        info!(target: "node", "became FOLLOWER");
    }

    pub async fn stop(self: &Arc<Self>) {
        let mut state = self.state.write().await;
        if let Some(mut l) = state.leader.take() {
            l.stop().await;
        }
        state.role = Role::Unknown;
    }

    /// Used by tests to mock the follower URL.
    pub async fn replace_follower(&self, follower: Follower) {
        self.state.write().await.follower = follower;
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("addr", &self.addr)
            .field("version", &self.version)
            .finish()
    }
}
