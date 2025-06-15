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

pub fn initialize_spot_manager(config: KVQuantConfig) -> spot::SpotManager {
    log::info!("Initializing SpotManager with block size: {}, spot capacity: {}", config.block_size, config.spot_capacity);
    spot::SpotManager::new(config.spot_capacity)
}

/// Initializes the BlockManager with a specified block size
pub fn initialize_block_manager(block_size: usize) -> block::BlockManager {\
    // This function initializes the BlockManager with a specified block size
    log::info!("Initializing BlockManager with block size: {}", block_size);
    block::BlockManager::new(block_size)

}