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
use ndarray::Array1;
use rayon::prelude::*;
use crate::server::AgentFlowServer;


// Federated ANSS are used for collaborative filtering
#[derive(Serialize, Deserialize)]
pub struct FederatedANSS;

// Search for similar items using ANSS
// This implementation uses federated ANSS
// Each client has its own ANSS instance
impl FederatedANSS {
    pub async fn search(&self, server: &AgentFlowServer, query: &Array1<f32>, top_m: usize) -> Vec<u32> {
        let results: Vec<_> = server.clients.par_iter()
            .map(|entry| {
                let client = entry.value();
                client.fusion_anns.collaborative_filter(query, top_m / server.clients.len())
            })
            .collect();

        let mut candidates: Vec<u32> = results.into_iter().flatten().collect();
        candidates.sort();
        candidates.dedup();
        candidates.truncate(top_m);

        let ranked: Vec<_> = server.clients.par_iter()
            .map(|entry| {
                let client = entry.value();
                rt.block_on(client.fusion_anns.heuristic_rerank(query, candidates.clone())).unwrap()
            })
            .collect();

        ranked.into_iter().flatten().collect()
    }
}