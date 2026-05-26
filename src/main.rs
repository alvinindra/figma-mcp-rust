//! Binary entry point — parses CLI args, sets up logging, runs the election,
//! then serves MCP over stdio.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use clap::Parser;
use rmcp::{transport::io::stdio, ServiceExt};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use figma_mcp_rust::election::Election;
use figma_mcp_rust::handler::Handler;
use figma_mcp_rust::node::Node;

/// Build-time version, override with `cargo build --release` and the `CARGO_PKG_VERSION` env.
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)]
#[command(name = "figma-mcp-rust", version = VERSION)]
struct Cli {
    /// IP address to listen on (use 0.0.0.0 to accept remote connections).
    #[arg(long, default_value = "127.0.0.1")]
    ip: String,

    /// Port to listen on for the Figma plugin bridge.
    #[arg(long, default_value_t = 1994)]
    port: u16,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    init_logging();

    let cli = Cli::parse();
    let ip: IpAddr = cli
        .ip
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid IP address: {:?}", cli.ip))?;
    if !ip.is_loopback() {
        warn!(
            "binding to {} — server will be reachable from the network with no authentication",
            ip
        );
    }
    let addr = SocketAddr::new(ip, cli.port);

    let node = Arc::new(Node::new(addr, VERSION.into()));
    let mut election = Election::new(addr, node.clone());
    election.start().await?;
    info!(
        "Starting figma-mcp-rust {} (role: {})",
        VERSION,
        node.role_name().await
    );

    let handler = Handler::new(node.clone(), VERSION.into());

    // Graceful Ctrl-C handling, mirroring the Go server.
    let shutdown_node = node.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Shutting down...");
        }
        election.stop().await;
        shutdown_node.stop().await;
    });

    // Serve MCP over stdio. The future completes when the client disconnects.
    let (read, write) = stdio();
    let serving = handler.serve((read, write)).await?;
    let _ = serving.waiting().await;
    Ok(())
}

fn init_logging() {
    // Logs go to stderr so they don't pollute stdio MCP transport.
    let filter =
        EnvFilter::try_from_env("FIGMA_MCP_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(true)
        .with_level(true)
        .try_init();
}
