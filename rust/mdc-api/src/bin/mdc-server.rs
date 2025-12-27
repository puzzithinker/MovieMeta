//! Movie Data Capture API Server
//!
//! REST API and WebSocket server for MDC

use std::net::SocketAddr;
use clap::Parser;

#[derive(Parser)]
#[clap(name = "mdc-server")]
#[clap(about = "Movie Data Capture API Server", version)]
struct Args {
    /// Server host address
    #[clap(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[clap(short, long, default_value = "3000")]
    port: u16,

    /// Enable debug logging
    #[clap(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    if args.debug {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    // Parse socket address
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;

    tracing::info!("Starting Movie Data Capture API Server");
    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("WebSocket endpoint: ws://{}/ws/progress", addr);
    tracing::info!("");
    tracing::info!("API Endpoints:");
    tracing::info!("  GET  /health              - Health check");
    tracing::info!("  POST /api/jobs            - Create job");
    tracing::info!("  GET  /api/jobs            - List jobs");
    tracing::info!("  GET  /api/jobs/:id        - Get job details");
    tracing::info!("  POST /api/jobs/:id/cancel - Cancel job");
    tracing::info!("  POST /api/jobs/:id/retry  - Retry job");
    tracing::info!("  POST /api/scan            - Scan folder");
    tracing::info!("  GET  /api/config          - Get configuration");
    tracing::info!("  POST /api/config          - Update configuration");
    tracing::info!("  GET  /api/stats           - Get statistics");
    tracing::info!("");

    // Start server
    mdc_api::serve(addr).await?;

    Ok(())
}
