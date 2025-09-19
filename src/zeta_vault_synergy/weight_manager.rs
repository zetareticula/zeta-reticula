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
use std::collections::HashMap;
use crate::VaultConfig;
use async_trait::async_trait;


// BinaryWeightSet struct for binary weights
#[derive(Debug, Clone)]
pub struct BinaryWeightSet {
    pub data: Vec<u8>, // binary weights
}

#[derive(Error, Debug)]
pub enum WeightManagerError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
}

// WeightManager trait for weight management which extends the WeightManager trait
// binary weights are stored in a HashMap with the model id as the key
#[async_trait]
pub trait WeightManager: Send + Sync {
    async fn store_binary_weights(&self, model_id: &str, binary_set: BinaryWeightSet) -> Result<(), WeightManagerError>;
    async fn get_binary_weights(&self, model_id: &str) -> Option<BinaryWeightSet>;
    async fn get_all(&self) -> Vec<Vec<u8>>;
}

pub struct WeightManagerImpl {
    binary_weights: Arc<RwLock<HashMap<String, BinaryWeightSet>>>,
    node_id: usize,
}

impl WeightManagerImpl {
    pub async fn new(node_id: usize, _config: VaultConfig) -> Result<Self, WeightManagerError> {
        Ok(WeightManagerImpl {
            binary_weights: Arc::new(RwLock::new(HashMap::new())), // binary weights HashMap
            node_id, // node id
        })
    }

        // store binary weights in the HashMap
        async fn store_binary_weights_impl(&self, model_id: &str, binary_set: BinaryWeightSet) -> Result<(), WeightManagerError> {
        let key = format!("binary_weights_{}", model_id);
        self.binary_weights.write().await.insert(key.clone(), binary_set);
        log::info!("Stored binary weights for model {}", model_id);
        Ok(())
    }

        async fn get_binary_weights_impl(&self, model_id: &str) -> Option<BinaryWeightSet> {
        let key = format!("binary_weights_{}", model_id);
        self.binary_weights.read().await.get(&key).cloned()
    }
}

#[async_trait]
impl WeightManager for WeightManagerImpl {
    async fn store_binary_weights(&self, model_id: &str, binary_set: BinaryWeightSet) -> Result<(), WeightManagerError> {
        self.store_binary_weights_impl(model_id, binary_set).await
    }

    async fn get_binary_weights(&self, model_id: &str) -> Option<BinaryWeightSet> {
        self.get_binary_weights_impl(model_id).await
    }

    async fn get_all(&self) -> Vec<Vec<u8>> {
        let store = self.binary_weights.read().await;
        store.values().map(|w| w.data.clone()).collect()
    }
}