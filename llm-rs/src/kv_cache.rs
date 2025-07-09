use kvquant_rs::{initialize_kv_cache, LogStructuredKVCache, KVQuantConfig};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex, RwLock};
use dashmap::DashMap;
use crate::spot::SpotManager;
use crate::block::{DataBlock, BlockState};
use crate::MesolimbicSystem;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::collections::HashMap;
use crate::quantizer::{KVQuantConfig, KVQuantizer};
use crate::pb::kv_quant_service_server::KVQuantServiceServer;
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct KVCache {
    inner: Arc<LogStructuredKVCache>,
}

impl KVCache {
    pub fn new(sparsity: f32, priority_tokens: Vec<u32>) -> Self {
        let config = KVQuantConfig {
            block_size: 100,
            spot_capacity: 10,
            salience_threshold: 0.05, // replace todo with a sensible default or parameter
        };
        let inner = Arc::new(initialize_kv_cache(config));
        KVCache { inner }
    }

    pub fn update(&self, token_id: u32, layer: usize, value: f32, salience_score: f32, pointer: usize, bias: f32) {
        self.inner.update(token_id, value, salience_score, pointer, bias);
    }

    pub fn invalidate_low_salience(&self, salience_scores: &[(u32, f32)]) {
        if salience_scores.is_empty() {
            return;
        }

        salience_scores.iter().for_each(|&(token_id, salience)| {
            if salience < self.inner.salience_threshold {
                if let Some(spot) = self.inner.spots.get_spot(token_id as usize) {
                    spot.blocks.iter().for_each(|block| {
                        if block.data.contains_key(&token_id) {
                            block.unmap();
                            block.invalidate();
                        }
                    });
                }
            }
        });

        self.inner.invalidate_low_salience(salience_scores);
    }

    pub fn erase_full_spots(&self) {
        self.inner.spots.spots.iter().filter(|spot| spot.is_full).for_each(|spot| {
            self.inner.spots.erase_spot(spot.id);
        });

        self.inner.erase_full_spots();
    }
}

pub fn initialize_kv_cache(config: KVQuantConfig) -> LogStructuredKVCache {
    LogStructuredKVCache::new(config)
}

#[derive(Debug)]
enum KVCacheError {
    SpotNotFound(usize),
    BlockNotFound(usize, usize),
}

impl fmt::Display for KVCacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KVCacheError::SpotNotFound(spot_id) => write!(f, "Spot with ID {} not found", spot_id),
            KVCacheError::BlockNotFound(spot_id, block_id) => write!(f, "Block with ID {} in Spot {} not found", block_id, spot_id),
        }
    }
}
