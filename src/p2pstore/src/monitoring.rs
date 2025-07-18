use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use metrics_util::MetricKindMask;
use std::net::SocketAddr;
use tokio::sync::OnceCell;
use tracing::{error, info, Level};
use tracing_subscriber::{fmt, EnvFilter, prelude::*}; 

static METRICS_HANDLE: OnceCell<PrometheusHandle> = OnceCell::const_new();

/// Initialize the metrics and logging system
pub async fn init_monitoring(service_name: &str) -> anyhow::Result<()> {
    init_metrics()?;
    init_logging(service_name)?;
    Ok(())
}

/// Initialize Prometheus metrics exporter
fn init_metrics() -> anyhow::Result<()> {
    let recorder = PrometheusBuilder::new()
        .idle_timeout(
            MetricKindMask::HISTOGRAM,
            Some(std::time::Duration::from_secs(30)),
        )
        .install_recorder()?;

    METRICS_HANDLE.set(recorder).map_err(|_| anyhow::anyhow!("Metrics already initialized"))?;
    
    info!("Metrics system initialized");
    Ok(())
}

/// Initialize structured logging
fn init_logging(service_name: &str) -> anyhow::Result<()> {
    // Configure log level from environment or default to info
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Configure JSON logging for production
    let json_layer = fmt::layer()
        .json()
        .with_filter(env_filter);

    // Configure pretty logging for development
    let fmt_layer = fmt::layer()
        .pretty()
        .with_filter(EnvFilter::new("info"));

    // Use RUST_LOG=debug for development, or RUST_LOG=info for production
    tracing_subscriber::registry()
        .with(json_layer)
        .with(fmt_layer)
        .init();

    info!("Logging system initialized for service: {}", service_name);
    Ok(())
}

/// Start the metrics HTTP server
pub async fn start_metrics_server(addr: SocketAddr) -> anyhow::Result<()> {
    let handle = METRICS_HANDLE.get().ok_or_else(|| anyhow::anyhow!("Metrics not initialized"))?;
    
    let app = warp::path!("metrics")
        .map(move || handle.render())
        .with(warp::trace::request());

    info!("Starting metrics server on {}", addr);
    warp::serve(app).run(addr).await;
    
    Ok(())
}

/// Define custom metrics
pub mod metrics {
    use metrics::{histogram, increment_counter};
    use std::time::Instant;

    // Request metrics
    pub fn record_request(method: &str, path: &str, status: u16, start_time: Instant) {
        let duration = start_time.elapsed();
        
        increment_counter!("http_requests_total", 
            "method" => method.to_string(),
            "path" => path.to_string(),
            "status" => status.to_string()
        );
        
        histogram!("http_request_duration_seconds", 
            duration.as_secs_f64(),
            "method" => method.to_string(),
            "path" => path.to_string()
        );
    }

    // Network metrics
    pub fn record_network_event(event_type: &str, peer_id: &str) {
        increment_counter!("p2p_events_total", 
            "type" => event_type.to_string(),
            "peer_id" => peer_id.to_string()
        );
    }

    // Storage metrics
    pub fn record_storage_operation(operation: &str, size_bytes: u64) {
        increment_counter!("storage_operations_total", 
            "operation" => operation.to_string()
        );
        
        metrics::histogram!("storage_operation_bytes", 
            size_bytes as f64,
            "operation" => operation.to_string()
        );
    }
}
