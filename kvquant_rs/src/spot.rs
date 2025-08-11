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

use crate::block::DataBlock;
use crate::KVQuantConfig;
use dashmap::DashMap;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::atomic::AtomicBool;

// Re-export BlockState from block module for convenience
pub use crate::block::BlockState;

// Local BlockState to avoid conflict with block::BlockState
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpotBlockState {
    Free,
    Occupied,
    Invalidated,
}

// Spot represents a collection of data blocks in the cache
// Each spot can hold a fixed number of blocks and tracks whether it is full
// It provides methods to append new blocks, erase existing blocks, and manage the state of the spot
//                             
// The blocks within a spot are managed as a vector, and the spot can be marked as full when all blocks are occupied
// The `append_block` method attempts to add a new block with the given parameters, returning the block ID if successful
// If the spot is full, it returns None
// The `erase` method clears all blocks in the spot and resets its full state
// The `Spot` struct is serialized and deserialized for persistence



// Removed duplicate BlockState enum since we now have SpotBlockState

pub struct Spot {
    pub id: usize,
    pub blocks: Vec<DataBlock>,
    pub is_full: bool,
    pub capacity: usize,
}

impl Spot {
    pub fn new(id: usize, capacity: usize) -> Self {
        let blocks = (0..capacity)
            .map(|i| DataBlock::new(i, 1)) // Initialize with block_size=1 for now
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
        
        // Find the first free block and its index
        for (_i, block) in self.blocks.iter_mut().enumerate() {
            if let crate::block::BlockState::Free = block.state {
                // Store the block ID before writing to it
                let block_id = block.id;
                
                // Write to the block
                block.write(token_id, value, pointer, bias, 0, (0, vec![]));
                
                // Check if all blocks are now occupied by checking if any are still free
                let has_free = self.blocks.iter().any(|b| matches!(b.state, crate::block::BlockState::Free));
                self.is_full = !has_free;
                return Some(block_id);
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
    pub spots: DashMap<usize, Arc<Mutex<Spot>>>,
    pub spot_capacity: usize,
    pub block_capacity: usize,
    pub next_spot_id: AtomicUsize,
    pub is_full: AtomicBool,
}

impl SpotManager {
    pub fn new(spot_capacity: usize, block_capacity: usize) -> Self {
        let spots = DashMap::new();
        spots.insert(0, Arc::new(Mutex::new(Spot::new(0, spot_capacity))));
        SpotManager {
            spots,
            spot_capacity,
            block_capacity,
            next_spot_id: AtomicUsize::new(1),
            is_full: AtomicBool::new(false),
        }
    }

    pub fn append(&self, token_id: u32, value: f32, pointer: usize, bias: f32) -> Option<(usize, usize)> {
        // Try to find a spot with available space
        for entry in self.spots.iter() {
            let spot_id = *entry.key();
            if let Ok(mut spot) = entry.value().lock() {
                if let Some(block_id) = spot.append_block(token_id, value, pointer, bias) {
                    return Some((spot_id, block_id));
                }
            }
        }

        // If no spot has space and we're at capacity, return None
        if self.is_full.load(Ordering::Relaxed) {
            return None;
        }

        // Create a new spot
        let spot_id = self.next_spot_id.fetch_add(1, Ordering::SeqCst);
        let mut new_spot = Spot::new(spot_id, self.block_capacity);
        let block_id = match new_spot.append_block(token_id, value, pointer, bias) {
            Some(id) => id,
            None => return None, // Shouldn't happen for a new spot, but handle it safely
        };
        
        // Add the new spot to the manager
        self.spots.insert(spot_id, Arc::new(Mutex::new(new_spot)));
        
        // Update full status if needed
        if self.spots.len() >= self.spot_capacity {
            self.is_full.store(true, Ordering::Relaxed);
        }

        Some((spot_id, block_id))
    }

    pub fn erase_spot(&self, spot_id: usize) {
        if let Some(spot_arc) = self.spots.get(&spot_id) {
            let mut spot = spot_arc.lock().unwrap();
            spot.erase();
        }
    }

    pub fn get_spot(&self, spot_id: &usize) -> Option<Arc<Mutex<Spot>>> {
        self.spots.get(spot_id).map(|entry| entry.value().clone())
    }

    pub fn iter(&self) -> Vec<Arc<Mutex<Spot>>> {
        self.spots.iter().map(|entry| entry.value().clone()).collect()
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
            spots: SpotManager::new(config.spot_capacity, config.block_size),
            block_size: config.block_size,
            valid_bitmap: DashMap::new(),
            salience_threshold: config.salience_threshold,
            lock: Mutex::new(()),
        }
    }

    pub fn update(&self, token_id: u32, value: f32, salience_score: f32, pointer: usize, bias: f32) {
        // Only proceed if the salience score meets the threshold
        if salience_score >= self.salience_threshold {
            // Get a lock to ensure thread safety
            let _guard = self.lock.lock().unwrap();
            
            // Try to append the data to a spot and get the location
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
                    if let Some(spot_arc) = self.spots.spots.get(spot_id) {
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
        for spot_arc in self.spots.spots.iter() {
            let spot = spot_arc.value().lock().unwrap();
            if spot.is_full {
                self.spots.erase_spot(spot.id);
            }
        }
    }
}
