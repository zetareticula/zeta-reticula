use crate::spot::SpotManager;
use serde::{Serialize, Deserialize};
use dashmap::DashMap;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct LogStructuredKVCache {
    spots: SpotManager,
    block_size: usize,
    valid_bitmap: DashMap<(usize, usize), bool>,
    salience_threshold: f32,
    lock: Mutex<()>,
}

impl LogStructuredKVCache {
    pub fn new(config: KVQuantConfig) -> Self {
        LogStructuredKVCache {
            spots: SpotManager::new(config.spot_capacity),
            block_size: config.block_size,
            valid_bitmap: DashMap::new(),
            salience_threshold: 0.7,
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
                    if let Some(spot) = self.spots.spots.get(spot_id) {
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
        for spot in self.spots.spots.iter() {
            if spot.is_full {
                self.spots.erase_spot(spot.id);
            }
        }
    }
}