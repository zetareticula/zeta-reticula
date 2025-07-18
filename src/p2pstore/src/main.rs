//! Main entry point for the p2pstore service

mod monitoring;

use anyhow::Context;
use p2pstore::{KVCache, TransferEngine};
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, Level, instrument};
use monitoring::metrics;

#[tokio::main]
#[instrument]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize monitoring
    monitoring::init_monitoring("p2pstore")
        .await
        .context("Failed to initialize monitoring")?;

    // Start metrics server
    let metrics_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 9090);
    let metrics_handle = tokio::spawn(monitoring::start_metrics_server(metrics_addr));

    info!("Starting p2pstore service...");
    let start_time = std::time::Instant::now();

    // Initialize the transfer engine with metrics
    let transfer_engine = Arc::new(Mutex::new(
        TransferEngine::new()
            .await
            .context("Failed to initialize transfer engine")?,
    ));

    // Start the P2P network
    let peer_id = start_p2p_network(transfer_engine.clone())
        .await
        .context("Failed to start P2P network")?;

    // Record startup metrics
    metrics::record_network_event("startup", &peer_id);
    info!("p2pstore service started with peer ID: {}", peer_id);
    info!("Startup completed in {:?}", start_time.elapsed());

    // Handle shutdown signals
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
        _ = metrics_handle => {
            error!("Metrics server task exited unexpectedly");
        }
    }

    info!("Shutting down p2pstore service...");
    Ok(())
}

async fn start_p2p_engine(engine: Arc<Mutex<dyn TransferEngine>>) -> Result<(), Box<dyn Error>> {
    // TODO: Implement P2P network initialization
    // - Set up libp2p transport
    // - Configure network behavior
    // - Start network processing
    Ok(())
}
