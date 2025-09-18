pub mod cli;
pub mod config;
pub mod engine;
pub mod error;
pub mod memory;
pub mod model;
pub mod quantization;
pub mod tokenizer;
pub mod utils;

// Re-export commonly used types
pub use cli::{Args, Commands, PrecisionLevel};
pub use config::Config;
pub use engine::QuantizationEngine;
pub use error::{QuantizationError, Result};
pub use memory::MemoryTracker;
pub use model::{ModelLoader, ModelFormat};
pub use quantization::{QuantizationEngine as CoreQuantizer, QuantizedTensor};

// Re-export tokenizer for external use
pub use crate::tokenizer::TokenizerIntegration;

// Integration with Zeta Reticula components
pub use ns_router_rs as ns_router;
pub use agentflow_rs as agentflow;
pub use salience_engine as salience;
pub use kvquant_rs as kvquant;
