use serde::{Serialize, Deserialize};
use ndarray::{Array2, Array1};
use tonic::{transport::Channel, Request, Status};
use log;
use crate::quantizer::QuantizationResult;

// gRPC client for zeta-sidecar
mod pb {
    tonic::include_proto!("sidecar"); // Generated from zeta-sidecar/proto/sidecar.proto
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YoungTableau {
    data: Array2<f32>, // Sparse matrix representing the tableau
    salience_threshold: f32,
    vector_ids: Vec<String>,
    layer_ids: Vec<String>,
}

impl YoungTableau {
    pub fn new(dimensions: usize, salience_threshold: f32) -> Self {
        let data = Array2::zeros((dimensions, dimensions));
        YoungTableau {
            data,
            salience_threshold,
            vector_ids: Vec::new(),
            layer_ids: Vec::new(),
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