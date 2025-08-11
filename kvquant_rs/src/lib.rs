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

use std::net::SocketAddr;
use std::sync::Arc;
use dashmap::DashMap;
use tonic::{transport::Server, Request, Response, Status};
use log::{info, error, debug};
use serde::Serialize;
use thiserror::Error;

// Include the generated protobuf code and re-export the service traits and types
pub mod pb {
    tonic::include_proto!("sidecar");
    
    // Re-export the service traits and types for convenience
    pub use sidecar_service_server::{SidecarService, SidecarServiceServer};
    pub use sidecar_service_client::SidecarServiceClient;
}

// Declare modules
mod kvquant_config;
mod role_inferer;
mod mesolimbic_system;
mod kv_quantizer;
pub mod block;
pub mod spot;
pub mod kv_cache;
pub mod model;
pub mod inference;
pub mod tableaux;

// Re-export for backward compatibility
pub use pb::*;

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



// Re-export commonly used types
pub use crate::block::DataBlock;
pub use crate::spot::SpotManager;
pub use crate::kv_cache::LogStructuredKVCache;
pub use crate::kvquant_config::{KVQuantConfig, PrecisionLevel, QuantizationResult, QuantizationData};
pub use crate::role_inferer::{RoleInferer, RoleInferenceResult};
pub use crate::mesolimbic_system::{MesolimbicSystem, SalienceResult};
pub use crate::kv_quantizer::KVQuantizer;
pub use crate::pb::sidecar_service_client::SidecarServiceClient;

// Re-export configuration types for backward compatibility
pub mod config {
    pub use crate::kvquant_config::*;
}

// Re-export role inference types for backward compatibility
pub mod role_inference {
    pub use crate::role_inferer::*;
}

// Re-export mesolimbic system types for backward compatibility
pub mod mesolimbic {
    pub use crate::mesolimbic_system::*;
}

// This module provides the main functionality for the KVQuant service
// It includes the KVQuantizer, KVQuantService, and related configurations
// It also handles the initialization of various components like KVCache and SpotManager
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
    ) -> std::result::Result<Response<CacheResponse>, Status> {
        let inner_result = (|| -> Result<Response<CacheResponse>> {
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
        })();
        inner_result.map_err(|e| Status::internal(e.to_string()))
    }

    async fn update_cache(
        &self,
        request: Request<CacheUpdate>,
    ) -> std::result::Result<Response<UpdateResponse>, Status> {
        let inner_result = (|| -> Result<Response<UpdateResponse>> {
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
                debug!("Updating cache for vector_id: {} ({} bytes)", req.vector_id, req.data.len());
            }

            self.cache.insert(req.vector_id.clone(), req.data);

            let response = UpdateResponse {
                status: "OK".to_string(),
            };
            Ok(Response::new(response))
        })();
        inner_result.map_err(|e| Status::internal(e.to_string()))
    }
}

impl KVQuantService {
    /// Creates a new instance of KVQuantService with the given configuration
    pub fn new(config: Option<KVQuantConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        info!("Initializing KVQuantService with config: {:?}", config);
        
        Self {
            config: config.clone(),
            cache: DashMap::with_capacity(config.max_cache_items.min(1000)),
            metrics: ServiceMetrics::default(),
        }
    }

    /// Runs the KVQuantService gRPC server
    pub async fn run_service(addr: &str) -> Result<()> {
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
    }
}







/// Configuration for the KVQuant system
pub fn initialize_kv_cache(config: KVQuantConfig) -> kv_cache::LogStructuredKVCache {
    log::info!("Initializing kvquant-rs with block size: {}, spot capacity: {}", config.block_size, config.spot_capacity);
    kv_cache::LogStructuredKVCache::new(config)
}

pub fn initialize_spot_manager(config: KVQuantConfig) -> spot::SpotManager {
    log::info!("Initializing SpotManager with block size: {}, spot capacity: {}", config.block_size, config.spot_capacity);
    spot::SpotManager::new(config.spot_capacity, config.block_size)
}

/// Initializes the SpotManager with a specified block size
pub fn initialize_block_manager(block_size: usize) -> spot::SpotManager {
    // This function initializes the SpotManager with a specified block size
    log::info!("Initializing SpotManager with block size: {}", block_size);
    spot::SpotManager::new(block_size, block_size * 2) // Using block_size * 2 as spot capacity
}

pub fn initialize_mesolimbic_system() -> Arc<MesolimbicSystem> {
    log::info!("Initializing MesolimbicSystem");
    Arc::new(MesolimbicSystem::new())
}

pub fn initialize_role_inferer(outer_iterations: usize, inner_iterations: usize) -> Arc<RoleInferer> {
    // Default threshold value
    let threshold = 0.1;
    
    // Create a new RoleInferer with the given number of iterations
    let role_inferer = RoleInferer::with_iterations(
        threshold,
        outer_iterations,
        inner_iterations
    );
    
    // Log the initialization
    info!(
        "Initialized RoleInferer with {} outer and {} inner iterations",
        outer_iterations, inner_iterations
    );
    
    Arc::new(role_inferer)
}



pub fn initialize_quantization_result(
    token_id: u32, 
    precision: PrecisionLevel, 
    salience_score: f32, 
    row: usize, 
    role: String, 
    role_confidence: f32
) -> QuantizationResult<QuantizationData> {
    log::info!("Initializing QuantizationResult for token_id: {}, precision: {:?}, salience_score: {}, row: {}, role: {}, role_confidence: {}", 
        token_id, precision, salience_score, row, role, role_confidence
    );
    
    Ok(QuantizationData {
        token_id,
        precision,
        salience_score,
        row,
        role,
        role_confidence,
    })
}