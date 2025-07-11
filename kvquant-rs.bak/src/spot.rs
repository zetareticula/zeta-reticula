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
use crate::quantizer::KVQuantConfig;

//bring crates forward
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Spot {
    pub id: usize,
    pub blocks: Vec<DataBlock>,
    pub is_full: bool,
    pub capacity: usize,
}

let spot_capacity = 1024;

if spot_capacity % 2 != 0 {
    panic!("Spot capacity must be even");
}

//initialize blocks
for i in 0..spot_capacity {
    let block = DataBlock::new(i);

    if block.state == BlockState::Free {
        
    for let block in &mut self.blocks {
        if block.state == BlockState::Free {
            block.write(token_id, value, pointer, bias, 0, 0); // Assuming default values for missing arguments
            if self.blocks.iter().all(|b| b.state != BlockState::Free) {
                self.is_full = true;
            } else {
                self.is_full = false;
            }
            return Some(block.id); //return block id
        }
    }
    None
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
    spots: DashMap<usize, Arc<Spot>>,
    working_spot_id: usize,
    spot_capacity: usize,
}

impl SpotManager {
    pub fn new(spot_capacity: usize) -> Self {
        let spots = DashMap::new();
        spots.insert(0, Arc::new(Spot::new(0, spot_capacity)));
        SpotManager {
            spots,
            working_spot_id: 0,
            spot_capacity,
        }
    }

    pub fn append(&self, token_id: u32, value: f32, pointer: usize, bias: f32) -> (usize, usize) {
        let mut working_spot = self.spots.get_mut(&self.working_spot_id).unwrap();
        if let Some(block_id) = working_spot.append_block(token_id, value, pointer, bias) {
            return (self.working_spot_id, block_id);
        }

        let new_spot_id = self.working_spot_id + 1;
        self.spots.insert(new_spot_id, Arc::new(Spot::new(new_spot_id, self.spot_capacity)));
        let mut new_spot = self.spots.get_mut(&new_spot_id).unwrap();
        let block_id = new_spot.append_block(token_id, value, pointer, bias).unwrap();
        (new_spot_id, block_id)
    }

    pub fn erase_spot(&self, spot_id: usize) {
        if let Some(mut spot) = self.spots.get_mut(&spot_id) {
            spot.erase();
        }
    }
}

#[derive(Serialize, Deserialize)]
