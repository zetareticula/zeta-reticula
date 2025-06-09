use serde::{Serialize, Deserialize};
use log;

pub mod block;
pub mod spot;
pub mod kv_cache;

#[derive(Serialize, Deserialize)]
pub struct KVQuantConfig {
    pub block_size: usize,  // Tokens per block
    pub spot_capacity: usize,  // Blocks per spot
}

pub fn initialize_kv_cache(config: KVQuantConfig) -> kv_cache::LogStructuredKVCache {
    log::info!("Initializing kvquant-rs with block size: {}, spot capacity: {}", config.block_size, config.spot_capacity);
    kv_cache::LogStructuredKVCache::new(config)
}