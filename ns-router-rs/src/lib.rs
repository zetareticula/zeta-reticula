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


//! # NS Router
//! 
//! A neurosymbolic router for inference requests that combines neural and symbolic approaches
//! to optimize model selection and execution strategy.

#![warn(missing_docs)]

use log;
use serde::{Serialize, Deserialize};

// Re-export commonly used types
pub use shared::{QuantizationResult, PrecisionLevel};

// Export modules
pub mod context;
pub mod rewrite_wrapper;
pub mod router;
pub mod salience;
pub mod strategy;
pub mod symbolic;

// Re-export commonly used items
pub use context::{NSContextAnalysis, NSContextAnalyzer};
pub use router::{NSRouter, TokenFeatures};
pub use salience::SalienceAnalyzer;
pub use strategy::{ExecutionStrategy, ModelConfig, KVCacheConfig};

/// Neurosymbolic Routing Plan
/// 
/// Defines how an inference request should be routed, including model configuration,
/// execution strategy, and KV cache settings.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NSRoutingPlan {
    /// Configuration for the model to use
    pub model_config: ModelConfig,
    
    /// Strategy for executing the model
    pub execution_strategy: String,
    
    /// Configuration for the KV cache
    pub kv_cache_config: KVCacheConfig,
    
    /// Symbolic rules to apply during inference
    pub symbolic_rules: Vec<String>,
}

/// Initialize a new NSRouter instance
/// 
/// # Returns
/// A new instance of `NSRouter` ready to handle routing requests.
///
/// # Examples
/// ```
/// use ns_router_rs::initialize_ns_router;
///
/// let router = initialize_ns_router();
/// ```
pub fn initialize_ns_router() -> NSRouter {
    log::info!("Initializing NS Router");
    NSRouter::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_initialization() {
        let router = initialize_ns_router();
        assert!(router.route_inference("test input", "user1").is_ok());
    }
}