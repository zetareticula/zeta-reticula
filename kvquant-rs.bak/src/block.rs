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
use std::collections::HashMap;
use std::vec::Vec;
use std::sync::Arc;
use dashmap::DashMap;
use std::sync::Mutex;
use std::sync::RwLock;
use dashmap::mapref::entry::Entry;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicI32;
use crate::tableaux::YoungTableau;
use crate::role_inference::{RoleInferer, RoleInferenceResult};
use crate::mesolimbic::{MesolimbicSystem, SalienceResult};
use crate::role_inference::RoleTheory;
use crate::quantizer::{QuantizationResult, PrecisionLevel};
use crate::quantizer::KVQuantConfig;
use crate::spot::SpotManager;
use crate::block::{DataBlock, BlockState};
use crate::quantizer::KVQuantizer;


// KVQuantizer is the main structure for handling key-value quantization
// It manages data blocks, role inference, and the mesolimbic system for salience computation.

#[derive(Serialize, Deserialize, Clone)]



#[derive(Serialize, Deserialize)]
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
                precision: self.config.precision_level,
                salience_score: value * self.config.salience_threshold,
                row: block.id,
                role: "default".to_string(), // Placeholder for role
                role_confidence: 1.0, // Placeholder for confidence
            })
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct KVQuantConfig {
    pub block_size: usize,
    pub spot_capacity: usize,
    pub salience_threshold: f32,
    pub precision_level: PrecisionLevel,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum PrecisionLevel {
    Bit16,
    Bit32,
    Bit64,
}



#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum BlockState {
    Free,
    Valid,
    Obsolete,
    Invalid,
}

#[derive(Serialize, Deserialize)]
pub struct DataBlock {
    pub id: usize,
    pub state: BlockState,
    pub data: HashMap<u32, f32>,
    pub pointers: Vec<usize>,
    pub biases: Vec<f32>,
    pub vector_ids: Vec<u32>,  // ANNS vector-IDs
    pub navigation_graph: HashMap<usize, Vec<usize>>,  // Navigation graph entries
}

impl DataBlock {
    pub fn new(id: usize) -> Self {
        DataBlock {
            id,
            state: BlockState::Free,
            data: HashMap::new(),
            pointers: vec![],
            biases: vec![],
            vector_ids: vec![],
            navigation_graph: HashMap::new(),
        }
    }

    pub fn write(&mut self, token_id: u32, value: f32, pointer: usize, bias: f32, vector_id: u32, graph_entry: (usize, Vec<usize>)) {
        if self.state == BlockState::Free {
            self.data.insert(token_id, value);
            self.pointers.push(pointer);
            self.biases.push(bias);
            self.vector_ids.push(vector_id);
            self.navigation_graph.insert(graph_entry.0, graph_entry.1);
            self.state = BlockState::Valid;
        }
    }

    pub fn unmap(&mut self) {
        if self.state == BlockState::Valid {
            self.state = BlockState::Obsolete;
        }
    }

    pub fn invalidate(&mut self) {
        if self.state == BlockState::Obsolete {
            self.state = BlockState::Invalid;
        }
    }

    pub fn erase(&mut self) {
        self.data.clear();
        self.pointers.clear();
        self.biases.clear();
        self.vector_ids.clear();
        self.navigation_graph.clear();
        self.state = BlockState::Free;
    }
}

#[derive(Serialize, Deserialize)]
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
            config,
            spots: SpotManager::new(config.spot_capacity),
            valid_bitmap: DashMap::new(),
            lock: Arc::new(Mutex::new(())),
            salience_threshold: config.salience_threshold,
        }
    }

    pub fn update(&self, token_id: u32, value: f32, salience_score: f32, pointer: usize, bias: f32) {
        let _guard = self.lock.lock().unwrap();
        if salience_score < self.salience_threshold {
            return;
        }
        let (spot_id, block_id) = self.spots.append(token_id, value, pointer, bias);
        self.valid_bitmap.insert((spot_id, block_id), true);
    }

    pub fn invalidate_low_salience(&self, salience_scores: &[(u32, f32)]) {
        let _guard = self.lock.lock().unwrap();
        for &(token_id, salience) in salience_scores {
            if salience < self.salience_threshold {
                for entry in self.valid_bitmap.iter() {
                    let ((spot_id, block_id), _) = entry.pair();
                    if let Some(spot) = self.spots.get_spot(*spot_id) {
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
            if spot.is_full() {
                self.spots.erase_spot(spot.id);
            }
        }
    }
}

pub fn initialize_kv_cache(config: KVQuantConfig) -> LogStructuredKVCache {
    LogStructuredKVCache::new(config)
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct KVQuantConfig {
    pub block_size: usize,
    pub spot_capacity: usize,
    pub salience_threshold: f32,
    pub precision_level: PrecisionLevel,
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_kv_quantizer() {
        let config = KVQuantConfig {
            block_size: 10,
            spot_capacity: 100,
            salience_threshold: 0.5,
            precision_level: PrecisionLevel::Bit32,
        };
        let kv_quantizer = KVQuantizer::new(config);

        let token_id = 42;
        let value = 0.8;
        let pointer = 1;
        let bias = 0.1;
        let vector_id = 100;
        let graph_entry = (0, vec![1, 2, 3]);

        let result = kv_quantizer.quantize(token_id, value, pointer, bias, vector_id, graph_entry);
        assert!(result.is_some());
    }
}