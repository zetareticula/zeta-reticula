use serde::{Serialize, Deserialize};
use log;
use bumpalo::Bump;
use rayon::prelude::*;
use rand_distr::{Distribution, Normal};
use std::mem;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use neon::prelude::*;
use crate::block::DataBlock;
use crate::quantizer::{QuantizationResult, PrecisionLevel};
use crate::tableaux::YoungTableau;
use crate::role_inference::{RoleInferer, RoleInferenceResult};
use crate::mesolimbic::{MesolimbicSystem, SalienceResult};
use crate::role_inference::RoleTheory;
use crate::quantizer::KVQuantConfig;
use crate::quantizer::KVQuantizer;
use crate::pb::kv_quant_service_server::KVQuantServiceServer;
use crate::pb::{KVQuantRequest, KVQuantResponse};
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
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



// Ensure the tonic build script is executed
fn build() {
    tonic_build::compile_protos("proto/sidecar.proto").unwrap();
}

tonic_build::compile_protos("proto/sidecar.proto").unwrap(); // Ensure the proto files are compiled

mod pb {
    tonic::include_proto!("kvquant"); // The proto package name
    pub use kv_quant_service_server::KVQuantServiceServer;
}
    pub use kv_quant_service_client::KVQuantServiceClient;
    pub use kv_quant_service::KVQuantService;
}

pub struct KVQuantService {
    config: Option<HashMap<String, String>>, // Assuming config is a HashMap
}

impl KVQuantService {
    pub fn new(config: Option<HashMap<String, String>>) -> Self {
        let mut block_size = 1024; // Default value
        if let Some(ref config) = config {
            if let Some(value) = config.get("block_size") {
                block_size = value.parse().unwrap();
            }
        }
        let quantizer = KVQuantizer::new(KVQuantConfig { block_size, spot_capacity: todo!(), salience_threshold: todo!() });
        KVQuantService { config }
    }


    pub fn run_service(service: KVQuantService) {
        if let Err(e) = service.start() {
            log::error!("Failed to start KVQuantService: {}", e);
        }
    }
}




// KVQuantizer is the main structure for handling key-value quantization
#[derive(Serialize, Deserialize, Clone)]
pub struct KVQuantizer {
    pub config: KVQuantConfig,
    pub data_blocks: DashMap<usize, DataBlock>, // Concurrent access to data blocks
    pub role_inferer: Arc<RoleInferer>,
    pub mesolimbic_system: Arc<MesolimbicSystem>,
}

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