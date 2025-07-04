use kvquant_rs::{initialize_kv_cache, LogStructuredKVCache, KVQuantConfig};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use crate ::spot::SpotManager;
use crate::block::{DataBlock, BlockState};
use crate::MesolimbicSystem;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::collections::HashMap;
use std::sync::RwLock;
use crate::quantizer::KVQuantConfig;
use crate::quantizer::KVQuantizer;
use crate::pb::kv_quant_service_server::KVQuantServiceServer;


// This module provides the main functionality for the KVCache\
// It includes the KVCache struct, which manages a log-structured key-value cache
// It also handles the initialization of various components like LogStructuredKVCache and SpotManager
#[derive(Serialize, Deserialize)]
pub struct KVCache {
    inner: LogStructuredKVCache,

}

impl KVCache {
    pub fn new(sparsity: f32, priority_tokens: Vec<u32>) -> Self {
        let config = KVQuantConfig {
            block_size: 100,
            spot_capacity: 10,
            salience_threshold: todo!(),
        };
        let inner = initialize_kv_cache(config);
        KVCache { inner }
    }

    pub fn update(&self, token_id: u32, layer: usize, value: f32, salience_score: f32, pointer: usize, bias: f32) {
        self.inner.update(token_id, value, salience_score, pointer, bias);
    }

    pub fn invalidate_low_salience(&self, salience_scores: &[(u32, f32)]) {
        if (salience_scores.is_empty()) {
            return;
        }

        for &(token_id, salience) in salience_scores {
            if salience < self.inner.salience_threshold {
                if let Some(spot) = self.inner.spots.get_spot(token_id as usize) {
                    for block in spot.blocks.iter() {
                        if block.data.contains_key(&token_id) {
                            block.unmap();
                            block.invalidate();
                        }
                    }
                }
            }
        }
        self.inner.invalidate_low_salience(salience_scores);
    }

    pub fn erase_full_spots(&self) {
        for spot in self.inner.spots.spots.iter() {
            if spot.is_full {
                self.inner.spots.erase_spot(spot.id);
            }
        }
        // Erase full spots from the inner cache
        self.inner.erase_full_spots();
    }
}


pub fn initialize_kv_cache(config: KVQuantConfig) -> LogStructuredKVCache {
    LogStructuredKVCache::new(config)
}


enum KVCacheError {
    SpotNotFound(usize),
    BlockNotFound(usize, usize),
}

impl std::fmt::Display for KVCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KVCacheError::SpotNotFound(spot_id) => write!(f, "Spot with ID {} not found", spot_id),
            KVCacheError::BlockNotFound(spot_id, block_id) => write!(f, "Block with ID {} in Spot {} not found", block_id, spot_id),
        }
    }
}