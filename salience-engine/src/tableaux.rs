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

use ndarray::Array2;
use tonic::{transport::Channel, Request, Status};
use log;
use crate::quantizer::QuantizationResult;
use serde::{Serialize, Deserialize};
use ndarray::s;
use ndarray::ArrayViewMut2;
use std::sync::Arc;
use zeta_vault::ZetaVault;
use zeta_vault::VaultConfig;



// This module implements the Young Tableau structure for managing salience quantization results
// and caching them to a sidecar service for further processing.

// This attribute allows the module inception warning to be ignored
// It should be placed at the top of the file, before any items
// This module implements the Young Tableau structure for managing salience quantization results
// and caching them to a sidecar service for further processing.


// gRPC client for zeta-sidecar
pub mod pb {
    tonic::include_proto!("pb"); // The string specified here must match the proto package name
}

// Ensure you have a build.rs file with the following content:
// FILEPATH: /Users/xaxpmore/Documents/GitHub/zeta-reticula/salience-engine/build.rs
fn main() {
    tonic_build::compile_protos("proto/sidecar.proto").unwrap();
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YoungTableau {
    pub(crate) data: Array2<f32>, // Sparse matrix representing the tableau
    pub(crate) salience_threshold: f32,
    pub(crate) vector_ids: Vec<String>,
    pub(crate) layer_ids: Vec<String>,
    pub(crate) dimensions: (usize, usize),
    pub(crate) threshold: f32,
    pub(crate) rows: Vec<Vec<_>>,
}

impl YoungTableau {
    pub fn new(dimensions: usize, salience_threshold: f32) -> Self {
        let data = Array2::zeros((dimensions, dimensions));
        YoungTableau {
            data,
            salience_threshold,
            vector_ids: Vec::new(),
            layer_ids: Vec::new(),
            dimensions: todo!(),
            threshold: todo!(),
            rows: todo!(),
        }
    }

    pub fn from_quantization_results(results: &[QuantizationResult], dimensions: usize) -> Self {
        let mut tableau = YoungTableau::new(dimensions, 0.7); // Default threshold
        let mut data = Array2::zeros((dimensions, dimensions));

        for (i, result) in results.iter().enumerate() {
            if result.salience_score > tableau.salience_threshold {
                let row = i % dimensions;
                let col = (i / dimensions) % dimensions;
                data[[row, col]] = result.salience_score;
                tableau.vector_ids.push(format!("vec_{}", result.token_id));
            }
        }

        tableau.data = data;
        tableau
    }

    pub fn sparsify(&mut self) {
        // Apply sparsity based on salience threshold
        for ((i, j), value) in self.data.indexed_iter_mut() {
            if *value < self.salience_threshold {
                *value = 0.0;
            }
        }
        self.vector_ids.retain(|_| {
            let idx = self.vector_ids.iter().position(|v| v == _).unwrap();
            self.data[[idx % self.data.dim().0, idx / self.data.dim().0]] != 0.0
        });
    }

    pub async fn cache_to_sidecar(&self, sidecar_client: &mut pb::sidecar_service_client::SidecarServiceClient<Channel>) -> Result<(), Status> {
        for (i, vector_id) in self.vector_ids.iter().enumerate() {
            let layer_id = self.layer_ids.get(i).unwrap_or(&"layer_001".to_string());
            let data = self.data.slice(s![i, ..]).to_vec().into_raw_vec();
            let request = Request::new(pb::CacheUpdate {
                vector_id: vector_id.clone(),
                data: data.into(),
            });
            let response = sidecar_client.update_cache(request).await?;
            log::info!("Cached {} to sidecar: {}", vector_id, response.into_inner().status);
        }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tableau_sparsification() {
        let mut tableau = YoungTableau::new(3, 0.5);
        tableau.data[[0, 0]] = 0.7;
        tableau.data[[1, 1]] = 0.3;
        tableau.vector_ids = vec!["vec_001".to_string(), "vec_002".to_string()];
        tableau.sparsify();
        assert_eq!(tableau.data[[0, 0]], 0.7);
        assert_eq!(tableau.data[[1, 1]], 0.0);
        assert_eq!(tableau.vector_ids.len(), 1);
    }
}