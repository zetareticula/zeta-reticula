use serde::{Serialize, Deserialize};
use log;

pub mod router;
pub mod strategy;
pub mod context;

#[derive(Serialize, Deserialize)]
pub struct RoutingPlan {
    pub model_config: ModelConfig,
    pub execution_strategy: String,
    pub kv_cache_config: KVCacheConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ModelConfig {
    pub size: usize,          // e.g., 3B, 7B parameters
    pub precision: Vec<PrecisionLevel>,
}

#[derive(Serialize, Deserialize)]
pub struct KVCacheConfig {
    pub sparsity: f32,        // 0.0 to 1.0 (fraction of cache retained)
    pub priority_tokens: Vec<u32>,
}

#[derive(Serialize, Deserialize)]
pub enum PrecisionLevel {
    Bit4,
    Bit8,
    Bit16,
}

pub fn initialize_router() -> router::Router {
    log::info!("Initializing router-rs");
    router::Router::new()
}