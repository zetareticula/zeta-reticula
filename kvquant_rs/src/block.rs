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
use std::sync::Arc;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::spot::SpotManager;
use crate::{KVQuantConfig, PrecisionLevel, QuantizationResult, QuantizationData, RoleInferer, MesolimbicSystem};

// This module provides the main functionality for the KVQuantizer
#[derive(Clone, Debug)]
pub struct KVQuantizer {
    pub config: KVQuantConfig,
    pub data_blocks: DashMap<usize, DataBlock>,
    role_inferer: Arc<RoleInferer>,
    mesolimbic_system: Arc<MesolimbicSystem>,
}

impl PartialEq for KVQuantizer {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
            && self.role_inferer.threshold == other.role_inferer.threshold
            && self.mesolimbic_system.learning_rate == other.mesolimbic_system.learning_rate
            && self.mesolimbic_system.discount_factor == other.mesolimbic_system.discount_factor
    }
}

impl Eq for KVQuantizer {}

impl KVQuantizer {
    /// Create a new KVQuantizer with the given configuration
    pub fn new(config: KVQuantConfig) -> Self {
        Self {
            config: config.clone(),
            data_blocks: DashMap::new(),
            role_inferer: Arc::new(RoleInferer::default()),
            mesolimbic_system: Arc::new(MesolimbicSystem::default()),
        }
    }

    /// Allocate a new data block
    pub fn allocate_block(&self, id: usize) -> DataBlock {
        if self.data_blocks.contains_key(&id) {
            if self.data_blocks.get(&id).unwrap().state == BlockState::Free {
                return self.data_blocks.get(&id).unwrap().clone();
            }
        }
        for block in self.data_blocks.iter() {
            if block.value().state == BlockState::Free {
                return block.value().clone();
            }
        }
        if self.data_blocks.len() >= self.config.block_size {
        //eviction
        let mut block = self.data_blocks.iter().min_by_key(|b| b.value().access_count).unwrap();
        block.value().invalidate();
        block.value().erase();
        self.data_blocks.remove(&block.key());

        
        }

        DataBlock::new(id, self.config.block_size)
        
    }

    /// Get a reference to a data block by ID
    pub fn get_block(&self, id: usize) -> Option<DataBlock> {

        if let mut block = self.data_blocks.get_mut(&id) {
            block.value().access_count += 1;
            block.value().last_accessed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            return Some(block.value().clone());
        }
        None
    }

    /// Update the access tracking for a block
    pub fn update_access_tracking(&self, block_id: usize) {
        if let mut block = self.data_blocks.get_mut(&block_id) {
            block.value().access_count += 1;
            block.value().last_accessed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
    }

    /// Insert or update a data block
    pub fn insert_block(&self, block: DataBlock) {
        self.data_blocks.insert(block.id, block);
    }

    /// Quantize a value with the given parameters
    pub fn quantize(&self, token_id: u32, value: f32, pointer: usize, bias: f32, vector_id: u32, graph_entry: (usize, Vec<usize>)) -> Option<QuantizationResult<QuantizationData>> {
        let block_id = (token_id as usize) % self.config.block_size;
        let mut block = self.data_blocks.entry(block_id).or_insert_with(|| DataBlock::new(block_id, self.config.block_size));

        if block.state == BlockState::Free || block.state == BlockState::Valid {
            block.write(token_id, value, pointer, bias, vector_id, graph_entry);
            Some(Ok(QuantizationData {
                token_id,
                precision: self.config.precision,
                salience_score: value * self.config.salience_threshold,
                row: block.id,
                role: "default".to_string(),
                role_confidence: 1.0,
            }))
        } else {
            None
        }
    }
}

// Define basic types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockState {
    Free,
    Valid,
    Obsolete,
    Invalid,


}

// Define the data block structure for the KVQuantizer system
#[derive(Clone, Debug)]
pub struct DataBlock {
    pub id: usize,
    pub state: BlockState,
    pub data: HashMap<u32, f32>,
    pub pointers: Vec<usize>,
    pub biases: Vec<f32>,
    pub vector_ids: Vec<u32>,
    pub navigation_graph: HashMap<usize, Vec<usize>>,
    pub size: usize,
    pub capacity: usize,
}



impl DataBlock {
    /// Create a new data block with the given ID and capacity
    pub fn new(id: usize, capacity: usize) -> Self {
        Self {
            id,
            state: BlockState::Free,
            data: HashMap::new(),
            pointers: Vec::with_capacity(capacity),
            biases: Vec::with_capacity(capacity),
            vector_ids: Vec::with_capacity(capacity),
            navigation_graph: HashMap::new(),
            size: 0,
            capacity,
        }
    }

    /// Update the data block with new values
    pub fn update(&mut self, token_id: u32, value: f32, _salience_score: f32, pointer: usize, bias: f32) {
        // Update the data map
        self.data.insert(token_id, value);
        
        // Update the pointers and biases vectors if needed
        if let Some(idx) = self.vector_ids.iter().position(|&id| id == token_id) {
            // Update existing entry
            if idx < self.pointers.len() {
                self.pointers[idx] = pointer;
            }
            if idx < self.biases.len() {
                self.biases[idx] = bias;
            }
        } else if self.vector_ids.len() < self.capacity {
            // Add new entry if there's capacity
            self.vector_ids.push(token_id);
            self.pointers.push(pointer);
            self.biases.push(bias);
            
            // Update the navigation graph with an empty vector for the new entry
            self.navigation_graph.insert(self.vector_ids.len() - 1, Vec::new());
            
            // Update the size
            self.size = self.vector_ids.len();
        }
    }

    /// Write data to the block
    pub fn write(&mut self, token_id: u32, value: f32, pointer: usize, bias: f32, vector_id: u32, graph_entry: (usize, Vec<usize>)) {
        if self.state == BlockState::Free || self.state == BlockState::Valid {
            self.data.insert(token_id, value);
            self.pointers.push(pointer);
            self.biases.push(bias);
            self.vector_ids.push(vector_id);
            self.navigation_graph.insert(graph_entry.0, graph_entry.1);
            self.size += 1;
            self.state = BlockState::Valid;
        }
    }

    /// Unmap the block
    pub fn unmap(&mut self) {
        if self.state == BlockState::Valid {
            self.state = BlockState::Obsolete;
        }
    }

    /// Invalidate the block
    pub fn invalidate(&mut self) {
        if self.state == BlockState::Obsolete {
            self.state = BlockState::Invalid;
        }
    }

    /// Erase the block's contents
    pub fn erase(&mut self) {
        self.data.clear();
        self.pointers.clear();
        self.biases.clear();
        self.vector_ids.clear();
        self.navigation_graph.clear();
        self.size = 0;
        self.state = BlockState::Free;
    }
}

// Re-export types for backward compatibility
pub mod kvquant_rs {
    pub use crate::block::{DataBlock, BlockState};
    pub use crate::{
        KVQuantConfig, PrecisionLevel, QuantizationResult,
        RoleInferer, RoleInferenceResult,
        MesolimbicSystem, SalienceResult,
        KVQuantizer
    };
}


pub struct LogStructuredKVCache {
    pub config: KVQuantConfig,
    pub spots: SpotManager,
    pub valid_bitmap: DashMap<(usize, usize), bool>, // (spot_id, block_id)
    pub lock: Arc<Mutex<()>>,
    pub salience_threshold: f32,
}

impl LogStructuredKVCache {
    pub fn new(config: KVQuantConfig) -> Self {
        LogStructuredKVCache {
            config: config.clone(),
            spots: SpotManager::new(config.spot_capacity, config.block_size),
            valid_bitmap: DashMap::new(),
            lock: Arc::new(Mutex::new(())),
            salience_threshold: config.salience_threshold,
        }
    }

    pub fn update(&self, token_id: u32, value: f32, salience_score: f32, pointer: usize, bias: f32) {
        // Only proceed if the salience score meets the threshold
        if salience_score >= self.salience_threshold {
            // Get a lock to ensure thread safety
            let _guard = self.lock.lock().unwrap();
            
            // Append the data to a spot and get the location
            if let Some((spot_id, block_id)) = self.spots.append(token_id, value, pointer, bias) {
                // Mark the location as valid
                self.valid_bitmap.insert((spot_id, block_id), true);
            }
        }
    }

    pub fn invalidate_low_salience(&self, salience_scores: &[(u32, f32)]) {
        let _guard = self.lock.lock().unwrap();
        for &(token_id, salience) in salience_scores {
            if salience < self.salience_threshold {
                for entry in self.valid_bitmap.iter() {
                    let ((spot_id, block_id), _) = entry.pair();
                    if let Some(spot_arc) = self.spots.get_spot(spot_id) {
                        let mut spot = spot_arc.lock().unwrap();
                        if spot.blocks[*block_id].data.contains_key(&token_id) {
                            spot.blocks[*block_id].unmap();
                            spot.blocks[*block_id].invalidate();
                            self.valid_bitmap.insert((*spot_id, *block_id), false);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn erase_full_spots(&self) {
        let _guard = self.lock.lock().unwrap();
        for spot in self.spots.iter() {
            let spot = spot.lock().unwrap();
            if spot.is_full {
                self.spots.erase_spot(spot.id);
            }
        }
    }
}

pub fn initialize_kv_cache(config: KVQuantConfig) -> LogStructuredKVCache {
    LogStructuredKVCache::new(config)
}

pub struct KVCache {
    pub inner: LogStructuredKVCache,
}

impl KVCache {
    pub fn new(config: KVQuantConfig) -> Self {
        let inner = initialize_kv_cache(config);
        KVCache { inner }
    }

    pub fn update(&self, token_id: u32, value: f32, salience_score: f32, pointer: usize, bias: f32) {
        self.inner.update(token_id, value, salience_score, pointer, bias);
    }

    pub fn invalidate_low_salience(&self, salience_scores: &[(u32, f32)]) {
        self.inner.invalidate_low_salience(salience_scores);
    }

    pub fn erase_full_spots(&self) {
        self.inner.erase_full_spots();
    }
}
