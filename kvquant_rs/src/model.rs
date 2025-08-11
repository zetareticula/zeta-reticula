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


/*
Zeta Reticula is a Rust library for working with KVQuant models. It provides functionality to manage key-value caches, inference, and model interactions.
*/

use ndarray::{Array2, Array1};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::marker::PhantomData;
use crate::kvquant_config::{KVQuantConfig, PrecisionLevel, QuantizationResult, QuantizationData, QuantizationDataTrait};
use crate::role_inferer::RoleInferer;
use crate::mesolimbic_system::MesolimbicSystem;
// Re-export the client for use in other modules
pub use crate::pb::sidecar_service_client::SidecarServiceClient;

#[derive(Serialize, Deserialize)]
pub struct KVQuantModel<T: QuantizationDataTrait> {
    pub matrix: Array2<f32>,  // Preallocated FFN matrix (up + down project)
    pub pointers: Vec<usize>, // Original neuron indices
    pub bias: Array1<f32>,   // Bias for up project
    pub num_used: usize,     // Number of active rows
    pub last_k_active: Vec<usize>,  // Last k active neuron indices
    pub precision_config: Vec<PrecisionLevel>,
    pub predictor: RoleInferer,
    pub chunk_size: usize,   // 32KiB chunks
    pub d_model: usize,      // Model dimension
    _phantom: PhantomData<T>,
}

impl<T: QuantizationDataTrait> KVQuantModel<T> {
    pub fn new(size: usize, quantization_results: &[QuantizationResult<T>]) -> Self {
        let d_model = 768;  // Example dimension (adjust based on model)
        let req_i = size / d_model;  // Max neurons from validation set
        let matrix = Array2::zeros((req_i, 2 * d_model));  // Preallocated matrix
        let pointers = vec![0; req_i];
        let bias = Array1::zeros(req_i);
        KVQuantModel {
            matrix,
            pointers,
            bias,
            num_used: 0,
            last_k_active: vec![],
            precision_config: quantization_results.iter().map(|r| r.as_ref().map(|data| data.precision()).unwrap_or(PrecisionLevel::Int8)).collect(),
            predictor: RoleInferer::with_iterations(1.0, 10, 5), // threshold: 1.0, 10 outer, 5 inner iterations
            chunk_size: 32 * 1024,
            d_model,
            _phantom: PhantomData,
        }
    }

    pub fn load_from_flash(&mut self, file_path: &str) -> io::Result<()> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; self.chunk_size];
        while let Ok(n) = reader.read(&mut buffer) {
            if n == 0 { break; }
            // Process the chunk (e.g., deserialize or load into the model)
        }
        Ok(())
    }
}

    // pub fn predict_active_neurons(&self, preactivations: &Array1<f32>) -> Vec<bool> {
    //     self.predictor.predict_active_neurons(preactivations)
    // }

// KVQuantService is the gRPC service for KVQuant operations
#[derive(Default)]
pub struct KVQuantService {
    pub config: Option<HashMap<String, String>>,
    // pub kv_cache: Arc<RwLock<KVQuantCache>>, // Removed: KVQuantCache not defined
    pub role_inferer: Arc<RoleInferer>,
    pub mesolimbic_system: Arc<MesolimbicSystem>,
}

