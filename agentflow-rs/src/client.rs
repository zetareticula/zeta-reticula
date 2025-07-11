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