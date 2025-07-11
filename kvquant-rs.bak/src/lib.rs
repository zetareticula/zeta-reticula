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

// KVQuantizer is the main structure for handling key-value quantization
#[derive(Serialize, Deserialize, Clone)]
pub struct KVQuantizer {
    pub config: KVQuantConfig,
    pub data_blocks: DashMap<usize, DataBlock>, // Concurrent access to data blocks
    pub role_inferer: Arc<RoleInferer>,
    pub mesolimbic_system: Arc<MesolimbicSystem>,
    pub spot_manager: Arc<SpotManager>,
    pub kv_cache: Arc<KVCache>,
    pub block_manager: Arc<BlockManager>,

    if spot_manager.is_full() {
        spot_manager.erase_spot(spot_manager.working_spot_id);
        for block in &mut spot_manager.spots {
            block.erase();
        }
        spot_manager.working_spot_id = 0;
        spot_manager.spots.insert(0, Arc::new(Spot::new(0, spot_manager.spot_capacity)));
    }
        
    
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

#[derive(Serialize, Deserialize)]
pub struct KVQuantConfig {
    pub block_size: usize,  // Tokens per block
    pub spot_capacity: usize,  // Blocks per spot
}

pub fn initialize_kv_cache(config: KVQuantConfig) -> kv_cache::LogStructuredKVCache {
    let mut kv_cache = kv_cache::LogStructuredKVCache::new(config);
    for block in &mut kv_cache.blocks {
        block.erase();
    }

    let spot_manager = initialize_spot_manager(config);
    let block_manager = initialize_block_manager(config.block_size);
    let role_inferer = initialize_role_inferer(config.block_size, config.spot_capacity);
    let mesolimbic_system = initialize_mesolimbic_system();

    if spot_manager.is_full() {
        spot_manager.erase_spot(spot_manager.working_spot_id);
        for block in &mut spot_manager.spots {
            block.erase();
        }
        spot_manager.working_spot_id = 0;
        spot_manager.spots.insert(0, Arc::new(Spot::new(0, spot_manager.spot_capacity)));
    }

    if block_manager.is_full() {
        block_manager.erase();
    }

    let kv_quantizer = KVQuantizer {
        config,
        data_blocks: DashMap::new(),
        role_inferer,
        mesolimbic_system,
        spot_manager,
        block_manager,
    };

    //initialize kv_cache
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
    log::info!("Initializing RoleInferer with outer iterations: {}, inner iterations: {}", outer_iterations, inner_iterations);
    Arc::new(RoleInferer::new(outer_iterations, inner_iterations))
}

pub fn initialize_young_tableau(dimensions: usize, threshold: f32) -> YoungTableau {
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