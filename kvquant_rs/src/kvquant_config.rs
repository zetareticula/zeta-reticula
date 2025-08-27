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


//! Configuration types for KVQuant

use serde::{Deserialize, Serialize};

/// Configuration for KVQuant
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KVQuantConfig {
    /// Size of each block in the cache
    pub block_size: usize,
    /// Capacity of each spot
    pub spot_capacity: usize,
    /// Salience threshold for caching
    pub salience_threshold: f32,
    /// Precision level for quantization
    pub precision: PrecisionLevel,
    /// Enable debug logging
    pub enable_debug_logging: bool,
    /// Maximum number of items in cache
    pub max_cache_items: usize,
}

impl Default for KVQuantConfig {
    fn default() -> Self {
        Self {
            block_size: 4096,
            spot_capacity: 8,
            salience_threshold: 0.8,
            precision: PrecisionLevel::Int8,
            enable_debug_logging: false,
            max_cache_items: 1000,
        }
    }
}

/// Precision level for quantization
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrecisionLevel {
    /// 8-bit quantization
    Int8,
    /// 4-bit quantization
    Int4,
    /// 2-bit quantization
    Int2,
    /// 1-bit quantization
    Bit1,
}

/// Trait for quantization data that provides access to precision information
pub trait QuantizationDataTrait {
    /// Get the precision level used for quantization
    fn precision(&self) -> PrecisionLevel;
}

/// Data structure for quantization results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationData {
    /// Token ID
    pub token_id: u32,
    /// Precision level used for quantization
    pub precision: PrecisionLevel,
    /// Salience score of the token
    pub salience_score: f32,
    /// Row index in the matrix
    pub row: usize,
    /// Role of the token
    pub role: String,
    /// Confidence of the role assignment
    pub role_confidence: f32,
}

impl QuantizationDataTrait for QuantizationData {
    fn precision(&self) -> PrecisionLevel {
        self.precision
    }
}

/// Result type for quantization operations
pub type QuantizationResult<T> = Result<T, QuantizationError>;

/// Error type for quantization operations
#[derive(Debug, thiserror::Error)]
pub enum QuantizationError {
    /// Invalid input data
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    
    /// Unsupported precision level
    #[error("Unsupported precision level")]
    UnsupportedPrecision,
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Other errors
    #[error("Quantization error: {0}")]
    Other(String),
}
