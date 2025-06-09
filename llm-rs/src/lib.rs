use serde::{Serialize, Deserialize};
use log;

pub mod inference;
pub mod model;
pub mod kv_cache;
pub mod utils;

#[derive(Serialize, Deserialize)]
pub struct InferenceOutput {
    pub text: String,
    pub tokens_processed: usize,
    pub latency_ms: f32,
}

pub fn initialize_engine(model_size: usize) -> inference::InferenceEngine {
    log::info!("Initializing llm-rs with model size: {} parameters", model_size);
    inference::InferenceEngine::new(model_size)
}