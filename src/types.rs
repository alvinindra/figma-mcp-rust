//! Wire-format types shared between server, plugin (over WebSocket), and follower↔leader RPC.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Request sent from the server to the Figma plugin over WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRequest {
    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "requestId")]
    pub request_id: String,

    #[serde(rename = "nodeIds", default, skip_serializing_if = "Vec::is_empty")]
    pub node_ids: Vec<String>,

    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub params: Map<String, Value>,
}

/// Response received from the Figma plugin over WebSocket.
///
/// The plugin also emits progress frames during long-running calls: when `progress > 0`
/// the entry is kept in the pending map and the timeout is extended.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BridgeResponse {
    #[serde(rename = "type", default)]
    pub r#type: String,

    #[serde(rename = "requestId", default)]
    pub request_id: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub error: String,

    #[serde(default, skip_serializing_if = "is_zero")]
    pub progress: u32,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub message: String,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// Wire format for follower → leader /rpc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub tool: String,
    #[serde(rename = "nodeIds", default, skip_serializing_if = "Vec::is_empty")]
    pub node_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub params: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RpcResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub error: String,
}

/// Role of the current process.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Role {
    Unknown,
    Leader,
    Follower,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Leader => "LEADER",
            Role::Follower => "FOLLOWER",
            Role::Unknown => "UNKNOWN",
        }
    }
}
