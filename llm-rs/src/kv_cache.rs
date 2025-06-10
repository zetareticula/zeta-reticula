use kvquant_rs::{initialize_kv_cache, LogStructuredKVCache, KVQuantConfig};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct KVCache {
    inner: LogStructuredKVCache,
}

impl KVCache {
    pub fn new(sparsity: f32, priority_tokens: Vec<u32>) -> Self {
        let config = KVQuantConfig {
            block_size: 100,
            spot_capacity: 10,
        };
        let inner = initialize_kv_cache(config);
        KVCache { inner }
    }

    pub fn update(&self, token_id: u32, layer: usize, value: f32, salience_score: f32, pointer: usize, bias: f32) {
        self.inner.update(token_id, value, salience_score, pointer, bias);
    }

    pub fn invalidate_low_salience(&self, salience_scores: &[(u32, f32)]) {
        self.inner.invalidate_low_salience(salience_scores);
    }

    pub fn erase_full_spots(&self) {
        self.inner.erase_full_spots();
    }
}