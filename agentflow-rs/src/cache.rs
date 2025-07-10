use serde::{Serialize, Deserialize};
use crate::server::AgentFlowServer;
use std::sync::Arc;
use kvquant_rs::LogStructuredKVCache;

#[derive(Serialize, Deserialize)]
pub struct DistributedCache;

impl DistributedCache {
    pub fn update(&self, server: &AgentFlowServer, token_id: u32, value: f32, salience_score: f32, pointer: usize, bias: f32, vector_id: u32, graph_entry: (usize, Vec<usize>)) {
        let client_id = token_id as usize % server.clients.len();
        if let Some(client) = server.clients.get(&client_id) {
            client.kv_cache.update(token_id, value, salience_score, pointer, bias, vector_id, graph_entry);
        }
    }

    pub fn invalidate_low_salience(&self, server: &AgentFlowServer, salience_scores: &[(u32, f32)]) {
        server.clients.par_iter().for_each(|entry| {
            let client = entry.value();
            client.kv_cache.invalidate_low_salience(salience_scores);
        });
    }

    pub fn erase_full_spots(&self, server: &AgentFlowServer) {
        server.clients.par_iter().for_each(|entry| {
            let client = entry.value();
            client.kv_cache.erase_full_spots();
        });
    }
}