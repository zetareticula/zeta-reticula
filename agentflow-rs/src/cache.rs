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