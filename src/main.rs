//! Binary entry point — parses CLI args, sets up logging, runs the election,
//! then serves MCP over stdio.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use clap::Parser;
use rmcp::ServiceExt;
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
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Shutting down...");
                election.stop().await;
                shutdown_node.stop().await;
            }
            // Signal registration failed — keep serving rather than
            // tearing the bridge down at startup.
            Err(e) => warn!("ctrl-c handler unavailable: {e}"),
        }
    });

    // Serve MCP over stdio. The future completes when the client disconnects.
    // rmcp 0.3 tears down the session on a malformed JSON-RPC line, so filter
    // stdin first: unparsable lines are dropped and the loop continues, matching
    // the official SDKs (https://github.com/modelcontextprotocol/rust-sdk/issues/938).
    let (read, write) = (filtered_stdin(), tokio::io::stdout());
    let serving = handler.serve((read, write)).await?;
    let _ = serving.waiting().await;
    Ok(())
}

/// Reader that forwards stdin to the MCP transport line by line, dropping any
/// line that is not valid JSON so one garbled frame can't kill the session.
fn filtered_stdin() -> tokio::io::DuplexStream {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (mut tx, rx) = tokio::io::duplex(1024 * 1024);
    tokio::spawn(async move {
        let mut lines = BufReader::new(tokio::io::stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if serde_json::from_str::<serde::de::IgnoredAny>(&line).is_err() {
                warn!("ignoring unparsable JSON-RPC frame from stdin");
                continue;
            }
            if tx.write_all(line.as_bytes()).await.is_err()
                || tx.write_all(b"\n").await.is_err()
            {
                break;
            }
        }
        // tx drops here → EOF on the transport → clean shutdown.
    });
    rx
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
