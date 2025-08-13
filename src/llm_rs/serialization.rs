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

//! Model serialization and deserialization support

use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use ndarray::{Array2, Array3};
use crate::llm_rs::{LLMModel, AttentionWeights};

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    
    #[error("Invalid model format")]
    InvalidFormat,
}

#[derive(Serialize, Deserialize)]
struct ModelWeights {
    embed_dim: usize,
    layer_count: usize,
    weights: Vec<LayerWeights>,
}

#[derive(Serialize, Deserialize)]
struct LayerWeights {
    w_q: Vec<f32>,
    w_k: Vec<f32>,
    w_v: Vec<f32>,
    w_o: Vec<f32>,
    w_gate: Option<Vec<f32>>,
}

impl LLMModel {
    /// Save the model to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), SerializationError> {
        let weights = ModelWeights {
            embed_dim: self.embed_dim,
            layer_count: self.layer_count,
            weights: self.attention_weights.iter().map(|w| LayerWeights {
                w_q: w.w_q.clone().into_raw_vec(),
                w_k: w.w_k.clone().into_raw_vec(),
                w_v: w.w_v.clone().into_raw_vec(),
                w_o: w.w_o.clone().into_raw_vec(),
                w_gate: w.w_gate.as_ref().map(|g| g.clone().into_raw_vec()),
            }).collect(),
        };
        
        let bytes = bincode::serialize(&weights)?;
        let mut file = File::create(path)?;
        file.write_all(&bytes)?;
        
        Ok(())
    }
    
    /// Load a model from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, SerializationError> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        
        let weights: ModelWeights = bincode::deserialize(&bytes)?;
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        
        let attention_weights = weights.weights.into_iter().map(|w| {
            let shape = (weights.embed_dim, weights.embed_dim);
            AttentionWeights {
                w_q: Array2::from_shape_vec(shape, w.w_q).unwrap(),
                w_k: Array2::from_shape_vec(shape, w.w_k).unwrap(),
                w_v: Array2::from_shape_vec(shape, w.w_v).unwrap(),
                w_o: Array2::from_shape_vec(shape, w.w_o).unwrap(),
                w_gate: w.w_gate.map(|g| Array2::from_shape_vec(shape, g).unwrap()),
            }
        }).collect();
        
        Ok(Self {
            embed_dim: weights.embed_dim,
            layer_count: weights.layer_count,
            attention_weights,
            config: Default::default(),
            rng,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_model_serialization() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_model.bin");
        
        // Create and save model
        let model = LLMModel::new(12, 768).unwrap();
        model.save(&path).unwrap();
        
        // Load and verify
        let loaded = LLMModel::load(&path).unwrap();
        assert_eq!(model.embed_dim, loaded.embed_dim);
        assert_eq!(model.layer_count, loaded.layer_count);
    }
}
