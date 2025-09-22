// Copyright 2025 ZETA RETICULA INC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Shared types and utilities for Zeta Reticula
//! 
//! This module consolidates common types from multiple crates

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Re-export core types
pub use zeta_kv_cache::{KVCacheConfig, KVCacheError, PrecisionLevel as KVPrecisionLevel};
pub use zeta_quantization::{QuantizationConfig, QuantizationError, PrecisionLevel, QuantizationResult};
pub use zeta_salience::{SalienceConfig, SalienceError, SalienceResult, MesolimbicState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZetaConfig {
    pub kv_cache: KVCacheConfig,
    pub quantization: QuantizationConfig,
    pub salience: SalienceConfig,
    pub runtime: RuntimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub max_memory_mb: usize,
    pub worker_threads: usize,
    pub enable_gpu: bool,
    pub batch_size: usize,
    pub timeout_seconds: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 4096,
            worker_threads: 8,
            enable_gpu: false,
            batch_size: 32,
            timeout_seconds: 300,
        }
    }
}

impl Default for ZetaConfig {
    fn default() -> Self {
        Self {
            kv_cache: KVCacheConfig::default(),
            quantization: QuantizationConfig::default(),
            salience: SalienceConfig::default(),
            runtime: RuntimeConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub tokens_processed: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub quantization_ratio: f32,
    pub avg_salience: f32,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub parameters: u64,
    pub precision: PrecisionLevel,
    pub created_at: String,
}

pub type Result<T> = std::result::Result<T, ZetaError>;

#[derive(Debug, thiserror::Error)]
pub enum ZetaError {
    #[error("KV Cache error: {0}")]
    KVCache(#[from] KVCacheError),
    #[error("Quantization error: {0}")]
    Quantization(#[from] QuantizationError),
    #[error("Salience error: {0}")]
    Salience(#[from] SalienceError),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
}
