// Mock zeta-vault-synergy module for compilation
pub mod kv_cache_manager;

pub use kv_cache_manager::{KVCacheManager, KVCacheManagerImpl, KVCacheManagerError, KVCache, CacheLayer};

#[derive(Debug, Clone)]
pub struct VaultConfig {
    pub cache_size: usize,
    pub enable_compression: bool,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            cache_size: 1000,
            enable_compression: true,
        }
    }
}
