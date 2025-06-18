use crate::block::{DataBlock, BlockState};
use serde::{Serialize, Deserialize};
use dashmap::DashMap;
use std::sync::Arc;

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