// Copyright 2025 ZETA RETICULA
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

use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeightManagerError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
}

pub struct WeightManager {
    binary_weights: Arc<RwLock<HashMap<String, BinaryWeightSet>>>,
    node_id: usize,
}

impl WeightManager {
    pub async fn new(node_id: usize, config: VaultConfig) -> Result<Self, WeightManagerError> {
        Ok(WeightManager {
            binary_weights: Arc::new(RwLock::new(HashMap::new())),
            node_id,
        })
    }

    pub async fn store_binary_weights(&self, model_id: &str, binary_set: BinaryWeightSet) -> Result<(), WeightManagerError> {
        let key = format!("binary_weights_{}", model_id);
        self.binary_weights.write().await.insert(key.clone(), binary_set);
        log::info!("Stored binary weights for model {}", model_id);
        Ok(())
    }

    pub async fn get_binary_weights(&self, model_id: &str) -> Option<BinaryWeightSet> {
        let key = format!("binary_weights_{}", model_id);
        self.binary_weights.read().await.get(&key).cloned()
    }
}