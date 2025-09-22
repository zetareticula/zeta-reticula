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

//! Inference module for KVQuant-RS.
//! Handles loading and managing model matrix and active neuron prediction.

use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::marker::PhantomData;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};

use crate::kvquant_config::{KVQuantConfig, PrecisionLevel, QuantizationResult, QuantizationData, QuantizationDataTrait};
use crate::role_inferer::RoleInferer;

#[derive(Serialize, Deserialize)]
pub struct KVQuantModel<T: QuantizationDataTrait> {
    pub matrix: Array2<f32>,               // FFN matrix (up + down projection)
    pub pointers: Vec<usize>,              // Neuron index mapping
    pub bias: Array1<f32>,                 // Up projection bias
    pub num_used: usize,                   // Active rows count
    pub last_k_active: Vec<usize>,         // Recently used neurons
    pub precision_config: Vec<PrecisionLevel>,
    #[serde(skip)]                         // Predictor can't be serialized
    pub predictor: Option<RoleInferer>,
    pub chunk_size: usize,
    pub d_model: usize,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T: QuantizationDataTrait> KVQuantModel<T> {
    pub fn new(size: usize, quantization_results: &[QuantizationResult<T>]) -> Self {
        let d_model = 768;
        let req_i = size / d_model;
        KVQuantModel {
            matrix: Array2::zeros((req_i, 2 * d_model)),
            pointers: vec![0; req_i],
            bias: Array1::zeros(req_i),
            num_used: 0,
            last_k_active: vec![],
            precision_config: quantization_results.iter().map(|r| r.as_ref().map(|data| data.precision()).unwrap_or(PrecisionLevel::Int8)).collect(),
            predictor: Some(RoleInferer::with_iterations(0.1, 1, 1)),
            chunk_size: 32 * 1024,
            d_model,
            _phantom: PhantomData,
        }
    }

    pub async fn load_from_flash<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(), Box<dyn std::error::Error>>
    where
        T: serde::de::DeserializeOwned,
    {
        let file = File::open(file_path).await?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).await?;

        let model: KVQuantModel<T> = bincode::deserialize(&buffer)?;

        // Only the serializable fields are copied
        self.matrix = model.matrix;
        self.pointers = model.pointers;
        self.bias = model.bias;
        self.num_used = model.num_used;
        self.last_k_active = model.last_k_active;
        self.precision_config = model.precision_config;
        self.chunk_size = model.chunk_size;
        self.d_model = model.d_model;

        // Reconstruct predictor if not serialized
        self.predictor = Some(RoleInferer::with_iterations(0.1, 1, 1));

        Ok(())
    }

    pub async fn load<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: serde::de::DeserializeOwned,
    {
        let file = File::open(file_path).await?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).await?;
        let mut model: KVQuantModel<T> = bincode::deserialize(&buffer)?;
        model.predictor = Some(RoleInferer::with_iterations(0.1, 1, 1));
        Ok(model)
    }
}