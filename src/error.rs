use thiserror::Error;

/// Top-level errors used across the crate.
#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("plugin not connected")]
    NotConnected,

    #[error("request timed out")]
    Timeout,

    #[error("websocket: {0}")]
    Ws(String),

    #[error("send: {0}")]
    Send(String),

    #[error("serialization: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("cancelled")]
    Cancelled,
}

#[derive(Debug, Error)]
pub enum LeaderError {
    #[error("port {0} already in use")]
    PortInUse(u16),
    #[error("listen error: {0}")]
    Listen(std::io::Error),
    #[error("hyper: {0}")]
    Hyper(String),
}
