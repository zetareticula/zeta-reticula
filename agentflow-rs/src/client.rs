use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use std::sync::Arc;
use dashmap::DashMap;
use llm_rs::fusion_anns::FusionANNS;
use kvquant::LogStructuredKVCache;

#[derive(Serialize, Deserialize)]
pub struct Client {
    id: usize,
    fusion_anns: FusionANNS,
    kv_cache: Arc<LogStructuredKVCache>,
    local_data: String,  // Path to local SSD
}

impl Client {
    pub fn new(id: usize, vector_dim: usize, batch_size: usize, kv_cache: Arc<LogStructuredKVCache>, local_data: String) -> Self {
        Client {
            id,
            fusion_anns: FusionANNS::new(vector_dim, batch_size),
            kv_cache,
            local_data,
        }
    }

    pub async fn initialize(&mut self) {
        self.fusion_anns.initialize().await;
    }
}