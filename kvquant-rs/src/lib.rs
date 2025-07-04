use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use dashmap::DashMap;
use tonic::{Request, Response, Status, transport::Server};
use async_trait::async_trait;
use log::{info, error, debug};
use serde::Serialize;
use thiserror::Error;

// Include the generated protobuf code
pub mod sidecar {
    tonic::include_proto!("sidecar");
}

// Re-export the service traits and types for convenience
pub use sidecar::{
    sidecar_service_server::{SidecarService, SidecarServiceServer},
    sidecar_service_client::SidecarServiceClient,
    CacheRequest, CacheResponse, CacheUpdate, UpdateResponse,
};

// Custom error type for the KVQuant service
#[derive(Error, Debug)]
pub enum KVQuantError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Tonic transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    
    #[error("Invalid configuration: {0}")]
    Config(String),
    
    #[error("Cache error: {0}")]
    Cache(String),
}

// Type alias for Result<T, KVQuantError>
pub type Result<T> = std::result::Result<T, KVQuantError>;
use crate::pb::KVQuantServiceClient;



// This module provides the main functionality for the KVQuant service
// It includes the KVQuantizer, KVQuantService, and related configurations
// It also handles the initialization of various components like KVCache, SpotManager, and BlockManager
// The KVQuantService is the main entry point for handling requests and managing the quantization process

pub struct KVQuantServiceServerImpl {
    service: KVQuantService,
}

impl KVQuantServiceServerImpl {
    pub fn new(service: KVQuantService) -> Self {
        KVQuantServiceServerImpl { service }
    }
}



// Include the generated protobuf code
pub mod sidecar {
    tonic::include_proto!("sidecar");
}

// Re-export the service traits and types for convenience
pub struct KVQuantService {
    /// Service configuration
    config: KVQuantConfig,
    /// In-memory cache for key-value storage
    cache: DashMap<String, Vec<u8>>,
    /// Metrics for monitoring
    metrics: ServiceMetrics,
}

/// Service metrics for monitoring
#[derive(Debug, Default)]
struct ServiceMetrics {
    total_requests: std::sync::atomic::AtomicU64,
    cache_hits: std::sync::atomic::AtomicU64,
    cache_misses: std::sync::atomic::AtomicU64,
}

/// Snapshot of service metrics for monitoring
#[derive(Debug, Clone, Serialize)]
pub struct ServiceMetricsSnapshot {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Current number of items in the cache
    pub cache_size: u64,
}

#[tonic::async_trait]
impl SidecarService for KVQuantService {
    async fn get_cached_data(
        &self,
        request: Request<CacheRequest>,
    ) -> Result<Response<CacheResponse>, Status> {
        self.metrics.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        let req = request.into_inner();
        let cache_key = format!("{}:{}", req.vector_id, req.layer_id);
        
        if self.config.enable_debug_logging {
            debug!("Looking up cache key: {}", cache_key);
        }
        
        match self.cache.get(&cache_key) {
            Some(data) => {
                self.metrics.cache_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                
                if self.config.enable_debug_logging {
                    debug!("Cache hit for key: {} ({} bytes)", cache_key, data.len());
                }
                
                let response = CacheResponse {
                    data: data.value().clone(),
                    status: "OK".to_string(),
                };
                Ok(Response::new(response))
            }
            None => {
                self.metrics.cache_misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                
                if self.config.enable_debug_logging {
                    debug!("Cache miss for key: {}", cache_key);
                }
                
                let response = CacheResponse {
                    data: Vec::new(),
                    status: "Not Found".to_string(),
                };
                Ok(Response::new(response))
            }
        }
    }

    async fn update_cache(
        &self,
        request: Request<CacheUpdate>,
    ) -> Result<Response<UpdateResponse>, Status> {
        self.metrics.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        let req = request.into_inner();
        
        // Check if we need to evict old entries to make space
        if self.cache.len() >= self.config.max_cache_items {
            // Simple eviction strategy: remove the first item
            if let Some(entry) = self.cache.iter().next() {
                let key = entry.key().clone();
                self.cache.remove(&key);
                
                if self.config.enable_debug_logging {
                    debug!("Evicted key from cache: {}", key);
                }
            }
        }
        
        if self.config.enable_debug_logging {
            debug!("Updating cache for vector_id: {} ({} bytes)", 
                  req.vector_id, req.data.len());
        }
        
        self.cache.insert(req.vector_id.clone(), req.data);
        
        let response = UpdateResponse {
            status: "OK".to_string(),
        };
        
        Ok(Response::new(response))
    }
}

impl KVQuantService {
    /// Creates a new instance of KVQuantService with the given configuration
    pub fn new(config: Option<KVQuantConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        info!("Initializing KVQuantService with config: {:?}", config);
        
        Self {
            config,
            cache: DashMap::with_capacity(config.max_cache_items.min(1000)),
            metrics: ServiceMetrics::default(),
        }
    }

    /// Runs the KVQuantService gRPC server
    pub async fn run_service(addr: &str) -> Result<(), KVQuantError> {
        let service = KVQuantService::new(None);
        let addr: SocketAddr = addr.parse()
            .map_err(|e| KVQuantError::Config(format!("Invalid address: {}", e)))?;
        
        info!("Starting KVQuantService on {}", addr);
        
        Server::builder()
            .add_service(SidecarServiceServer::new(service))
            .serve(addr)
            .await
            .map_err(KVQuantError::Transport)?;
        
        info!("KVQuantService shutdown complete");
        Ok(())
    }
    
    /// Returns the current cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
impl KVQuantizer {
    pub fn new(config: KVQuantConfig) -> Self {
        let role_inferer = Arc::new(RoleInferer::new(10, 5)); // 10 outer, 5 inner iterations
        let mesolimbic_system = Arc::new(MesolimbicSystem::new());
        KVQuantizer {
            config,
            data_blocks: DashMap::new(),
            role_inferer,
            mesolimbic_system,
        }
    }

    pub fn quantize(&self, token_id: u32, value: f32, pointer: usize, bias: f32, vector_id: u32, graph_entry: (usize, Vec<usize>)) -> Option<QuantizationResult> {
        let block_id = (token_id as usize) % self.config.block_size;
        let mut block = self.data_blocks.entry(block_id).or_insert_with(|| DataBlock::new(block_id));

        if block.state == BlockState::Free || block.state == BlockState::Valid {
            block.write(token_id, value, pointer, bias, vector_id, graph_entry);
            Some(QuantizationResult {
                token_id,
                precision: PrecisionLevel::Bit16,
                salience_score: 0.0, // Placeholder for actual salience score
                row: 0, // Placeholder for actual row index
                role: String::new(), // Placeholder for actual role
                role_confidence: 0.0, // Placeholder for actual confidence
            })
        } else {
            None
        }
    }
}

pub mod inference;
pub mod model;
pub mod block;
pub mod spot;
pub mod kv_cache;



/// Configuration for the KVQuant system
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KVQuantConfig {
    /// Maximum number of spots in the cache
    pub spot_capacity: usize,
    /// Size of each block in the cache
    pub block_size: usize,
    /// Threshold for salience score to consider a token valid
    pub salience_threshold: f32,
}

impl Default for KVQuantConfig {
    fn default() -> Self {
        Self {
            spot_capacity: 1000,
            block_size: 1024,
            salience_threshold: 0.7,
        }
    }
}

pub fn initialize_kv_cache(config: KVQuantConfig) -> kv_cache::LogStructuredKVCache {
    log::info!("Initializing kvquant-rs with block size: {}, spot capacity: {}", config.block_size, config.spot_capacity);
    kv_cache::LogStructuredKVCache::new(config)
}

pub fn initialize_spot_manager(config: KVQuantConfig) -> spot::SpotManager {
    log::info!("Initializing SpotManager with block size: {}, spot capacity: {}", config.block_size, config.spot_capacity);
    spot::SpotManager::new(config.spot_capacity)
}

/// Initializes the BlockManager with a specified block size
pub fn initialize_block_manager(block_size: usize) -> block::BlockManager {\
    // This function initializes the BlockManager with a specified block size
    log::info!("Initializing BlockManager with block size: {}", block_size);
    block::BlockManager::new(block_size)

}

pub fn initialize_mesolimbic_system() -> Arc<MesolimbicSystem> {
    log::info!("Initializing MesolimbicSystem");
    Arc::new(MesolimbicSystem::new())
}

pub fn initialize_role_inferer(outer_iterations: usize, inner_iterations: usize) -> Arc<RoleInferer> {

    if let Some(iterations) = outer_iterations.checked_sub(1) {
        if iterations == 0 {
            log::warn!("Outer iterations must be greater than 0, setting to 1");
            return Arc::new(RoleInferer::new(1, inner_iterations));
        }
    } else {
        log::error!("Invalid outer iterations provided, defaulting to 1");
        return Arc::new(RoleInferer::new(1, inner_iterations));
    }

    if let Some(inner_iterations) = inner_iterations.checked_sub(1) {
        if inner_iterations == 0 {
            log::warn!("Inner iterations must be greater than 0, setting to 1");
            return Arc::new(RoleInferer::new(outer_iterations, 1));
        }
    } else {
        log::error!("Invalid inner iterations provided, defaulting to 1");
        return Arc::new(RoleInferer::new(outer_iterations, 1));
    }

    log::info!("Initializing RoleInferer with outer iterations: {}, inner iterations: {}", outer_iterations, inner_iterations);

    Arc::new(RoleInferer::new(outer_iterations, inner_iterations))
}

pub fn initialize_young_tableau(dimensions: usize, threshold: f32) -> YoungTableau {
    if let Some(dimensions) = dimensions.checked_sub(1) {
        if dimensions == 0 {
            log::warn!("Dimensions must be greater than 0, setting to 1");
            return YoungTableau::new(1, threshold);
        }
    } else {
        log::error!("Invalid dimensions provided, defaulting to 1");
        return YoungTableau::new(1, threshold);
    }

    if threshold < 0.0 || threshold > 1.0 {
        log::warn!("Threshold must be between 0 and 1, setting to 0.5");
        return YoungTableau::new(dimensions, 0.5);
    }
    log::info!("Initializing YoungTableau with dimensions: {}, threshold: {}", dimensions, threshold);
    YoungTableau::new(dimensions, threshold)
}

pub fn initialize_quantization_result(token_id: u32, precision: PrecisionLevel, salience_score: f32, row: usize, role: String, role_confidence: f32) -> QuantizationResult {
    log::info!("Initializing QuantizationResult for token_id: {}, precision: {:?}, salience_score: {}, row: {}, role: {}, role_confidence: {}", token_id, precision, salience_score, row, role, role_confidence);
    QuantizationResult {
        token_id,
        precision,
        salience_score,
        row,
        role,
        role_confidence,

    }
}