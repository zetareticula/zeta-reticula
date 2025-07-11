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

use crate::spot::SpotManager;
use serde::{Serialize, Deserialize};
use dashmap::DashMap;
use std::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use rand::Rng;
use std::sync::RwLock;
use crate::block::{DataBlock, BlockState};
use dashmap::mapref::entry::Entry;
use rand_distr::{Distribution, Normal};
use crate::KVQuantConfig;

// LogStructuredKVCache implements a log-structured key-value cache
pub struct LogStructuredKVCache {
    spots: SpotManager,
    block_size: usize,
    valid_bitmap: DashMap<(usize, usize), bool>,
    salience_threshold: f32,
    lock: Mutex<()>,
}

// SpotManager is a manager for handling spots in the cache
impl LogStructuredKVCache {
    // Creates a new LogStructuredKVCache with the given configuration
    pub fn new(config: KVQuantConfig) -> Self {
        LogStructuredKVCache {
            spots: SpotManager::new(config.spot_capacity),
            block_size: config.block_size,
            valid_bitmap: DashMap::new(),
            salience_threshold: 0.7,
            lock: Mutex::new(()),
        }
    }

    // update adds a new token to the cache if its salience score is above the threshold
    pub fn update(&self, token_id: u32, value: f32, salience_score: f32, pointer: usize, bias: f32) {
        let _guard = self.lock.lock().unwrap();
        if salience_score < self.salience_threshold {
            return;
        }
        // Generate a random spot ID based on the token ID
        let (spot_id, block_id) = self.spots.append(token_id, value, pointer, bias);
        self.valid_bitmap.insert((spot_id, block_id), true);
    }

    pub fn invalidate_low_salience(&self, salience_scores: &[(u32, f32)]) {
        let _guard = self.lock.lock().unwrap();
        for &(token_id, salience) in salience_scores {
            if salience < self.salience_threshold {
                for entry in self.valid_bitmap.iter() {
                    let ((spot_id, block_id), _) = entry.pair();
                    if let Some(spot_arc) = self.spots.get_spot(spot_id) {
                        let mut spot = spot_arc.lock().unwrap();
                        if spot.blocks[*block_id].data.contains(&token_id) {
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
        for spot_arc in self.spots.iter() {
            let spot = spot_arc.lock().unwrap();
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
    inner: LogStructuredKVCache,
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



