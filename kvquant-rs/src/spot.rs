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

use crate::block::{DataBlock, BlockState};
use serde::{Serialize, Deserialize};
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicI32;
use dashmap::mapref::entry::Entry;
use crate::block::{DataBlock, BlockState};
use crate::KVQuantConfig;

// Spot represents a collection of data blocks in the cache
// Each spot can hold a fixed number of blocks and tracks whether it is full
// It provides methods to append new blocks, erase existing blocks, and manage the state of the spot
//                             
// The blocks within a spot are managed as a vector, and the spot can be marked as full when all blocks are occupied
// The `append_block` method attempts to add a new block with the given parameters, returning the block ID if successful
// If the spot is full, it returns None
// The `erase` method clears all blocks in the spot and resets its full state
// The `Spot` struct is serialized and deserialized for persistence



#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockState {
    Free,
    Occupied,
    Invalidated,
}

#[derive(Serialize, Deserialize)]
pub struct Spot {
    pub id: usize,
    pub blocks: Vec<DataBlock>,
    pub is_full: bool,
    pub capacity: usize,
}

impl Spot {
    pub fn new(id: usize, capacity: usize) -> Self {
        let blocks = (0..capacity)
            .map(|i| DataBlock::new(i))
            .collect();
        Spot {
            id,
            blocks,
            is_full: false,
            capacity,
        }
    }

    pub fn append_block(&mut self, token_id: u32, value: f32, pointer: usize, bias: f32) -> Option<usize> {
        if self.is_full {
            return None;
        }
        for block in &mut self.blocks {
            if block.state == BlockState::Free {
                block.write(token_id, value, pointer, bias, 0, 0); // Assuming default values for missing arguments
                if self.blocks.iter().all(|b| b.state != BlockState::Free) {
                    self.is_full = true;
                }
                return Some(block.id);
            }
        }
        None
    }

    pub fn erase(&mut self) {
        for block in &mut self.blocks {
            block.erase();
        }
        self.is_full = false;
    }
}

pub struct SpotConfig {
    pub spot_capacity: usize, // Maximum number of blocks in a spot
}

impl SpotConfig {
    pub fn new(spot_capacity: usize) -> Self {
        SpotConfig { spot_capacity }
    }
}



pub struct SpotManager {
    spots: DashMap<usize, Arc<Mutex<Spot>>>,

    working_spot_id: usize,
    spot_capacity: usize,
}

impl SpotManager {
    pub fn new(spot_capacity: usize) -> Self {
        let spots = DashMap::new();
        spots.insert(0, Arc::new(Mutex::new(Spot::new(0, spot_capacity))));
        SpotManager {
            spots,
            working_spot_id: 0,
            spot_capacity,
        }
    }

    pub fn append(&self, token_id: u32, value: f32, pointer: usize, bias: f32) -> (usize, usize) {
        let working_spot_arc = self.spots.get(&self.working_spot_id).unwrap();
        let mut working_spot = working_spot_arc.lock().unwrap();
        if let Some(block_id) = working_spot.append_block(token_id, value, pointer, bias) {
            return (self.working_spot_id, block_id);
        }

        let new_spot_id = self.working_spot_id + 1;
        self.spots.insert(new_spot_id, Arc::new(Mutex::new(Spot::new(new_spot_id, self.spot_capacity))));
        let new_spot_arc = self.spots.get(&new_spot_id).unwrap();
        let mut new_spot = new_spot_arc.lock().unwrap();
        let block_id = new_spot.append_block(token_id, value, pointer, bias).unwrap();
        (new_spot_id, block_id)
    }

    pub fn erase_spot(&self, spot_id: usize) {
        if let Some(spot_arc) = self.spots.get(&spot_id) {
            let mut spot = spot_arc.lock().unwrap();
            spot.erase();
        }
    }

    pub fn get_spot(&self, spot_id: &usize) -> Option<Arc<Mutex<Spot>>> {
        self.spots.get(spot_id).map(|entry| Arc::clone(&*entry))
    }

    pub fn iter(&self) -> Vec<Arc<Mutex<Spot>>> {
        self.spots.iter().map(|entry| Arc::clone(entry.value())).collect()
    }
}

pub struct LogStructuredKVCache {
    pub spots: SpotManager,
    pub block_size: usize,
    pub valid_bitmap: DashMap<(usize, usize), bool>, // (spot_id, block_id)
    pub salience_threshold: f32,
    lock: Mutex<()>,
}


impl LogStructuredKVCache {
    pub fn new(config: KVQuantConfig) -> Self {
        LogStructuredKVCache {
            spots: SpotManager::new(config.spot_capacity),
            block_size: config.block_size,
            valid_bitmap: DashMap::new(),
            salience_threshold: config.salience_threshold,
            lock: Mutex::new(()),
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
                    if let Some(spot_arc) = self.spots.spots.get(spot_id) {
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
        for spot_arc in self.spots.spots.iter() {
            let spot = spot_arc.value().lock().unwrap();
            if spot.is_full {
                self.spots.erase_spot(spot.id);
            }
        }
    }
}
