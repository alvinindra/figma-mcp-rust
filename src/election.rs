//! Leader election: try to bind the port, fall back to follower if a healthy
//! leader is already listening, periodically watch the leader, take over if it dies.

use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::follower::Follower;
use crate::node::Node;
use crate::types::Role;

pub struct Election {
    follower: Follower,
    node: Arc<Node>,
    cancel: CancellationToken,
    handle: Option<JoinHandle<()>>,
}

impl Election {
    pub fn new(addr: std::net::SocketAddr, node: Arc<Node>) -> Self {
        Self {
            follower: Follower::new(format!("http://{}", addr)),
            node,
            cancel: CancellationToken::new(),
            handle: None,
        }
    }

    /// Determine the initial role and start monitoring.
    pub async fn start(&mut self) -> std::io::Result<()> {
        self.determine_role().await?;
        let cancel = self.cancel.clone();
        let node = self.node.clone();
        let follower = self.follower.clone();
        self.handle = Some(tokio::spawn(async move {
            monitor(cancel, node, follower).await;
        }));
        Ok(())
    }

    pub async fn stop(&mut self) {
        self.cancel.cancel();
        if let Some(h) = self.handle.take() {
            let _ = h.await;
        }
    }

    async fn determine_role(&self) -> std::io::Result<()> {
        match self.node.become_leader().await {
            Ok(()) => Ok(()),
            Err(_) => {
                // Port taken — see if a healthy leader is already there.
                if self.follower.ping().await {
                    self.node.become_follower().await;
                    Ok(())
                } else {
                    info!(target: "election", "port taken but leader not responding — will retry");
                    Ok(())
                }
            }
        }
    }
}

async fn monitor(cancel: CancellationToken, node: Arc<Node>, follower: Follower) {
    loop {
        let jitter = {
            let mut rng = rand::thread_rng();
            rng.gen_range(3000..5000)
        };
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = tokio::time::sleep(Duration::from_millis(jitter)) => {}
        }

        match node.role().await {
            Role::Follower => {
                if !follower.ping().await {
                    info!(target: "election", "leader not responding, attempting takeover...");
                    if let Err(e) = node.become_leader().await {
                        warn!(target: "election", "takeover failed: {e}");
                    }
                }
            }
            Role::Unknown => {
                // Try again.
                match node.become_leader().await {
                    Ok(()) => {}
                    Err(_) => {
                        if follower.ping().await {
                            node.become_follower().await;
                        }
                    }
                }
            }
            Role::Leader => {
                // Nothing to do — we own the bridge.
            }
        }
    }
}
