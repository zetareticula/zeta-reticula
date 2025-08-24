use master_service::MasterService;
use std::error::Error;
use std::net::SocketAddr;
use std::time::Duration;
use log::{info, error, warn};
use std::process;
use tokio::signal;
use tokio::time::{sleep, timeout};

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
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down...");
        },
        _ = terminate => {
            info!("Received terminate signal, shutting down...");
        },
    }
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
                error!("Error cleaning up stale nodes: {}", e);
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
            eprintln!("Error loading configuration: {}", e);
            process::exit(1);
        }
    };
    
    // Initialize logging
    if let Err(e) = init_logging(&config.log_level) {
        eprintln!("Error initializing logging: {}", e);
        process::exit(1);
    }
    
    info!("Starting Zeta Reticula Master Service");
    info!("Configuration: {:#?}", config);
    
    // Create the master service
    let service = MasterService::new();
    
    // Start the cleanup task for stale nodes
    let cleanup_service = service.clone();
    let cleanup_interval = config.node_timeout_seconds / 2;
    tokio::spawn(async move {
        start_cleanup_task(cleanup_service, cleanup_interval, config.node_timeout_seconds).await;
    });
    
    // Start the HTTP server
    let server = MasterService::start_server(service, config.bind_addr);
    
    // Wait for server to start with a timeout
    let server_handle = tokio::spawn(server);
    
    // Wait for server task or shutdown signal
    tokio::select! {
        result = server_handle => {
            if let Err(e) = result {
                error!("Server error: {}", e);
                process::exit(1);
            }
        },
        _ = handle_shutdown_signal() => {
            info!("Shutting down gracefully...");
            // Add any cleanup logic here
        },
    }
    
    info!("Shutdown complete");
    Ok(())
}
