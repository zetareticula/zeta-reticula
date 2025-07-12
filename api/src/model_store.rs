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


use std::sync::Arc;
use tokio::sync::RwLock;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use rayon::prelude::*;
use half::f16;
use ndarray::{Array2, array};
use zeta_vault::{ZetaVault, VaultConfig, KVCache, CacheLayer, SecretStore};
use log;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelStoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

}

macro_rules! privileged_store {
    ($store:expr, $key:expr, $value:expr) => {
        #[cfg(feature = "enterprise")]
        {
            if let Some(store) = &$store {
                store.store($key, $value).await.unwrap_or_else(|e| log::error!("Secret store failed: {:?}", e));
            }
        }
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactionRequest {
    model_id: String,
    level: usize,
    data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactionResponse {
    status: String,
    compacted_data: Vec<u8>,
}

pub struct ModelStore {
    models: Arc<RwLock<Vec<String>>>,
    vault: ZetaVault,
    secret_store: Option<SecretStore>,
    chunk_size: usize,
    neuron_matrices: Arc<RwLock<Vec<NeuronMatrix>>>,
    compaction_queue: Arc<RwLock<Vec<CompactionRequest>>>, // CaaS queue
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NeuronMatrix {
    matrix: Array2<f16>,
    pointers: Vec<usize>,
    bias: Vec<f32>,
    num_used: usize,
    last_k_active: Vec<usize>,
}

impl ModelStore {
    pub async fn new() -> Self {
        ModelStore {
            models: Arc::new(RwLock::new(Vec::new())),
            vault: todo!(), // Need to initialize vault
            secret_store: todo!(), // Need to initialize secret_store
            chunk_size: 32 * 1024,
            neuron_matrices: Arc::new(RwLock::new(Vec::new())),
            compaction_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_model(&self, model_id: String, path: &str) {
        let mut models = self.models.write().await;
        models.push(model_id.clone());

        let weights = self.load_model_weights(path).await.unwrap_or_else(|e| {
            log::error!("Failed to load model weights: {:?}", e);
            Vec::new()
        });
        let d_model = 768;
        let req_i = 1024;
        let matrix = Array2::<f16>::zeros((req_i, 2 * d_model));
        let neuron_matrix = NeuronMatrix {
            matrix,
            pointers: Vec::with_capacity(req_i),
            bias: vec![0.0; req_i],
            num_used: 0,
            last_k_active: Vec::with_capacity(10),
        };
        let mut neuron_matrices = self.neuron_matrices.write().await;
        neuron_matrices.push(neuron_matrix);

        self.vault.store(format!("weights_{}", model_id), weights).await.unwrap_or_else(|e| log::error!("Vault store failed: {:?}", e));
    }

    pub async fn get_model_path(&self, model_id: &str) -> Option<String> {
        let models = self.models.read().await;
        models.iter().find(|&id| id == model_id).cloned()
    }

    pub async fn get_model_weights(&self, model_id: &str) -> Option<Vec<u8>> {
        self.vault.get(format!("weights_{}", model_id)).await
    }

    pub async fn get_neuron_matrix(&self, model_id: &str) -> Option<NeuronMatrix> {
        let models = self.models.read().await;
        let index = models.iter().position(|id| id == model_id)?;
        let neuron_matrices = self.neuron_matrices.read().await;
        neuron_matrices.get(index).cloned()
    }

    pub async fn update_neuron_matrix(&self, model_id: &str, new_neurons: Vec<(usize, Array2<f16>, f32)>) {
        let models = self.models.read().await;
        let index = models.iter().position(|id| id == model_id).unwrap();
        let mut neuron_matrices = self.neuron_matrices.write().await;
        let matrix = &mut neuron_matrices[index];

        let current_time = chrono::Utc::now().timestamp() as usize;
        let inactive = matrix.last_k_active.iter().enumerate()
            .filter(|(_, &t)| current_time - t > 100)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        for i in inactive.iter().rev() {
            if *i < matrix.num_used - 1 {
                matrix.matrix.swap_rows(*i, matrix.num_used - 1);
                matrix.pointers.swap(*i, matrix.num_used - 1);
                matrix.bias.swap(*i, matrix.num_used - 1);
            }
            matrix.num_used -= 1;
        }

        let start = matrix.num_used;
        for (i, (ptr, weights, bias)) in new_neurons.into_iter().enumerate() {
            let row = start + i;
            matrix.matrix.slice_mut(s![row, ..]).assign(&weights);
            matrix.pointers.push(ptr);
            matrix.bias.push(bias);
            matrix.last_k_active.push(current_time);
        }
        matrix.num_used = start + new_neurons.len();
    }

    pub async fn enqueue_compaction(&self, request: CompactionRequest) {
        let mut queue = self.compaction_queue.write().await;
        queue.push(request);
    }

    pub async fn process_compaction(&self) -> Option<CompactionResponse> {
        let mut queue = self.compaction_queue.write().await;
        if let Some(request) = queue.pop() {
            // Stateless compaction logic
            let compacted_data = self.compact_data(&request.data, request.level).await;
            Some(CompactionResponse {
                status: "Compacted".to_string(),
                compacted_data,
            })
        } else {
            None
        }
    }

    async fn load_model_weights(&self, path: &str) -> Result<Vec<u8>, ModelStoreError> {
        let mut file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len() as usize;

        let attention_size = file_size / 3;
        let mut attention_weights = vec![0u8; attention_size];
        file.read_exact(&mut attention_weights)?;

        let ffn_size = file_size - attention_size;
        let non_sparse_ratio = 0.03;
        let non_sparse_size = (ffn_size as f64 * non_sparse_ratio) as usize;
        let mut ffn_weights = Vec::with_capacity(non_sparse_size);
        (0..ffn_size).into_par_iter()
            .step_by(self.chunk_size)
            .map(|offset| {
                let mut chunk = vec![0u8; self.chunk_size.min(ffn_size - offset)];
                file.seek(SeekFrom::Start(attention_size as u64 + offset as u64))?;
                file.read_exact(&mut chunk)?;
                chunk
            })
            .collect_into_vec(&mut ffn_weights);

        let mut weights = Vec::with_capacity(attention_size + non_sparse_size);
        weights.extend_from_slice(&attention_weights[..attention_size]);
        weights.extend_from_slice(&ffn_weights[..non_sparse_size]);

        if weights.len() > attention_size + non_sparse_size {
            weights.truncate(attention_size + non_sparse_size);
        }

        let kv_cache = KVCache {
            key: bincode::serialize(&Array2::<f16>::zeros((1, 1)))?,
            value: bincode::serialize(&Array2::<f16>::zeros((1, 1)))?,
            layer: CacheLayer::HBM,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        privileged_store!(self.secret_store, format!("kv_cache_{}", Path::new(path).file_stem().unwrap().to_str().unwrap()), bincode::serialize(&kv_cache)?);

        Ok(weights)
    }

    async fn compact_data(&self, data: &[u8], level: usize) -> Vec<u8> {
        // Mock compaction: Sort and filter data based on level
        let mut compacted = data.to_vec();
        compacted.sort(); // Stateless sorting as an example
        if level > 0 {
            compacted.retain(|&x| x % (level as u8 + 1) == 0); // Level-based filtering
        }
        compacted
    }
}