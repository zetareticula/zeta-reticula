use crate::strategy::{ModelConfig, KVCacheConfig, PrecisionLevel};
use crate::context::{NSContextAnalysis, SalienceResult};
use crate::symbolic::SymbolicReasoner;
use crate::router::NSRouter;

use serde::{Serialize, Deserialize};
use log;
use bumpalo::Bump;
use rayon::prelude::*;

pub mod inference;
pub mod model;
pub mod router;
pub mod strategy;
pub mod context;
pub mod symbolic;

pub mod fusion_anns;

/// Neurosymbolic Routing Plan
/// This module defines the routing plan for neurosymbolic inference, 
/// 
#[derive(Serialize, Deserialize)]
pub struct NSRoutingPlan {
    pub model_config: ModelConfig,
    pub execution_strategy: String,
    pub kv_cache_config: KVCacheConfig,
    pub symbolic_rules: Vec<String>,  // Symbolic constraints applied
}

#[derive(Serialize, Deserialize)]
pub struct ModelConfig {
    pub size: usize,
    pub precision: Vec<PrecisionLevel>,
}

#[derive(Serialize, Deserialize)]
pub struct KVCacheConfig {
    pub sparsity: f32,
    pub priority_tokens: Vec<u32>,
}

#[derive(Serialize, Deserialize)]
pub enum PrecisionLevel {
    Bit4,
    Bit8,
    Bit16,
}

pub fn initialize_ns_router() -> router::NSRouter {
    log::info!("Initializing ns-router-rs with neurosymbolic capabilities");
    router::NSRouter::new()
}