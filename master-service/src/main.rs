// Copyright 2025 ZETA RETICULA INC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use master_service::MasterService;
use std::error::Error;
use std::net::SocketAddr;
use std::time::Duration;
use log::{info, error, warn};
use std::process;
use tokio::signal;
use tonic::transport::Server;
use master_service::proto::master_service_server::MasterServiceServer;

/// Configuration for the master service
#[derive(Debug)]
struct Config {
    bind_addr: SocketAddr,
    log_level: String,
    node_timeout_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:8080".parse().expect("Invalid default bind address"),
            log_level: "info".to_string(),
            node_timeout_seconds: 300, // 5 minutes
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    fn from_env() -> Result<Self, Box<dyn Error>> {
        let mut config = Config::default();
        
        if let Ok(addr) = std::env::var("BIND_ADDR") {
            config.bind_addr = addr.parse()?;
        }
        
        if let Ok(level) = std::env::var("LOG_LEVEL") {
            config.log_level = level.to_lowercase();
        }
        
        if let Ok(timeout) = std::env::var("NODE_TIMEOUT_SECONDS") {
            config.node_timeout_seconds = timeout.parse()?;
        }
        
        Ok(config)
    }
}

/// Initialize logging with the specified log level
fn init_logging(level: &str) -> Result<(), Box<dyn Error>> {
    let env = env_logger::Env::default()
        .default_filter_or(level);
        
    env_logger::Builder::from_env(env)
        .format_timestamp_millis()
        .format_module_path(false)
        .init();
        
    Ok(())
}

/// Handle OS signals for graceful shutdown
async fn handle_shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    info!("Shutting down gracefully...");
}

/// Background task to clean up stale nodes periodically
async fn start_cleanup_task(service: MasterService, interval_seconds: u64, max_age_seconds: u64) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));
    
    loop {
        interval.tick().await;
        
        match service.cleanup_stale_nodes(max_age_seconds).await {
            Ok(removed) if removed > 0 => {
                info!("Cleaned up {} stale nodes", removed);
            }
            Err(e) => {
                error!("Failed to clean up stale nodes: {}", e);
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration
    let config = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    // Initialize logging
    if let Err(e) = init_logging(&config.log_level) {
        eprintln!("Failed to initialize logging: {}", e);
        process::exit(1);
    }

    info!("Starting master service with config: {:?}", config);

    // Create the master service
    let service = MasterService::new();

    // Start the cleanup task
    let cleanup_interval = config.node_timeout_seconds / 6; // Clean up more frequently than the timeout
    let cleanup_service = service.clone();
    let _cleanup_handle = tokio::spawn(async move {
        start_cleanup_task(cleanup_service, cleanup_interval, config.node_timeout_seconds).await;
    });

    // Create the gRPC server
    let addr = config.bind_addr;
    let svc = MasterServiceServer::new(service);
    
    info!("Starting gRPC server on {}", addr);
    
    // Start the server with graceful shutdown
    Server::builder()
        .add_service(svc)
        .serve_with_shutdown(addr, async {
            handle_shutdown_signal().await;
        })
        .await?;

    info!("Master service stopped");
    
    info!("Shutdown complete");
    Ok(())
}
