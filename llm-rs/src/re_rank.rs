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

use ndarray::{Array1, Array2, s};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use dashmap::DashMap;
use serde::{Serialize, Deserialize};
use std::cmp::Ordering;
use crate::fusion_anns::FusionANNSConfig;



//FusionANNS struct for ANN 

pub struct FusionANNS {
    raw_vectors_path: PathBuf,
    pq_vectors: Array2<f32>,
    vector_ids: DashMap<usize, Vec<u32>>,
    navigation_graph: HashMap<usize, Vec<usize>>,
    vector_dim: usize,
    batch_size: usize,
}

impl FusionANNS {
    pub fn new(config: FusionANNSConfig) -> Self {
        let num_vectors = 1_000_000_000 / config.vector_dim;
        FusionANNS {
            raw_vectors_path: config.ssd_path,
            pq_vectors: Array2::zeros((num_vectors, config.vector_dim)),
            vector_ids: DashMap::new(),
            navigation_graph: HashMap::new(),
            vector_dim: config.vector_dim,
            batch_size: config.batch_size,
        }
    }

    pub async fn initialize(&mut self) {
        for i in 0..1000 {
            self.navigation_graph.insert(i, vec![(i + 1) % 1000, (i + 2) % 1000]);
            self.vector_ids.insert(i, vec![(i as u32) * 1000; 10]);
        }

        for ((i, j), value) in self.pq_vectors.indexed_iter_mut() {
            *value = (i * j) as f32 * 0.01;
        }
    }

    //Heuristic reranking
    pub async fn heuristic_rerank(&self, query: &Array1<f32>, candidates: Vec<u32>) -> Vec<u32> {
        //ranked set of ranks
        let mut ranked = vec![];
        let mut prev_accuracy = 0.0;
        let file = File::open(&self.raw_vectors_path).await.unwrap();
        let mut reader = BufReader::new(file);

        
        for batch in candidates.chunks(self.batch_size) {
            let mut raw_vectors = Array2::zeros((batch.len(), self.vector_dim));
            let mut buffer = vec![0u8; batch.len() * self.vector_dim * 4];

            reader.read_exact(&mut buffer).await.unwrap();
            for (i, chunk) in buffer.chunks(self.vector_dim * 4).enumerate() {
                for (j, val) in chunk.chunks(4).enumerate() {
                    if val.len() == 4 {
                        raw_vectors[[i, j]] = f32::from_le_bytes([val[0], val[1], val[2], val[3]]);
                    }
                }
            }

            let mut batch_results: Vec<(u32, f32)> = batch.iter().enumerate()
                .map(|(i, &id)| (id, raw_vectors.slice(s![i, ..]).dot(query)))
                .collect();

            batch_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
            ranked.extend(batch_results.iter().map(|&(id, _)| id));

            let current_accuracy: f32 = batch_results.iter().map(|&(_, dist)| dist).sum::<f32>() / batch.len() as f32;
            if (current_accuracy - prev_accuracy).abs() < 0.01 {
                break;
            }
            prev_accuracy = current_accuracy;
        }

        ranked
    }
}
