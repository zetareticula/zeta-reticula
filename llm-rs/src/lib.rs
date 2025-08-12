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

mod inference;
pub mod kv_cache;
mod fusion_anns;
mod metrics;
mod utils;

use std::time::Instant;
use std::sync::{Arc, RwLock};
use ndarray::{array, Array2, ArrayView2, s};
use half::f16;
use serde::{Deserialize, Serialize};
use log;
// use rayon::prelude::*; // Not needed when using sequential iterators
use ns_router_rs::NSRoutingPlan;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use std::io::{Error as IoError, ErrorKind};
use std::path::PathBuf;

pub fn get_default_inference_config() -> InferenceConfig {
    InferenceConfig {
        d_model: 768,
        max_neurons: 1024,
        chunk_size: 32 * 1024,
        precision: "f16".to_string(),
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct InferenceConfig {
    pub d_model: usize,
    pub max_neurons: usize,
    pub chunk_size: usize,
    pub precision: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceOutput {
    pub text: String,
    pub tokens_processed: usize,
    pub latency_ms: f64,
}

pub struct InferenceEngine {
    d_model: usize,
    weights: Vec<u8>,
    neuron_matrix: Option<Arc<RwLock<Array2<f16>>>>,
    num_used: usize,
}

impl InferenceEngine {
    pub async fn new(d_model: usize) -> Self {
        InferenceEngine {
            d_model,
            weights: Vec::new(),
            neuron_matrix: None,
            num_used: 0,
        }
    }

    pub async fn load_from_flash(&mut self, path: &str, is_enterprise: bool) -> Result<(), IoError> {
        let weights = tokio::fs::read(path).await?;
        self.weights = weights;

        if !is_enterprise {
            let req_i = 1024;
            let matrix = Array2::<f16>::from_elem((req_i, 2 * self.d_model), f16::from_f32(0.0));
            self.neuron_matrix = Some(Arc::new(RwLock::new(matrix)));
            self.num_used = 0;
        }
        Ok(())
    }

    pub async fn load_weights(&mut self, weights: Vec<u8>) {
        self.weights = weights;
    }

    pub async fn infer(
        &self,
        input: &str,
        routing_plan: &NSRoutingPlan,
        weights: &[u8],
    ) -> InferenceOutput {
        let start = Instant::now();
        let tokens: Vec<&str> = input.split_whitespace().collect();
        let tokens_processed = tokens.len();

        let matrix_lock = self.neuron_matrix.as_ref().unwrap();
        let matrix = matrix_lock.read().unwrap();
        let up_project = matrix.slice(s![..self.num_used, ..self.d_model]);
        let down_project = matrix.slice(s![..self.num_used, self.d_model..]).reversed_axes();

        let attention_scores: Vec<f16> = (0..self.num_used)
            .into_iter()
            .map(|i| {
                (0..self.d_model)
                    .map(|d| up_project[[i, d]] * down_project[[i, d]])
                    .fold(f16::from_f32(0.0), |acc, val| acc + val)
            })
            .collect();

        let text = format!("Processed: {}", input);
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        InferenceOutput {
            text,
            tokens_processed,
            latency_ms,
        }
    }

    pub async fn update_neuron_matrix(&mut self, new_neurons: Vec<(usize, Array2<f16>, f32)>) {
        if let Some(matrix_lock) = &self.neuron_matrix {
            let mut matrix = matrix_lock.write().unwrap();
            let current_time = chrono::Utc::now().timestamp() as usize;
            let k = 10;

            let inactive = (0..self.num_used)
                .filter(|&i| current_time - i > 100)
                .collect::<Vec<_>>();

            for i in inactive.iter().rev() {
                if *i < self.num_used - 1 {
                    // Manually swap rows by copying
                    let last_row = matrix.row(self.num_used - 1).to_owned();
                    let i_row = matrix.row(*i).to_owned();
                    matrix.slice_mut(s![self.num_used - 1, ..]).assign(&i_row);
                    matrix.slice_mut(s![*i, ..]).assign(&last_row);
                }
                self.num_used -= 1;
            }

            let start = self.num_used;
            for (i, (_ptr, weights, _bias)) in new_neurons.into_iter().enumerate() {
                let row = start + i;
                matrix.slice_mut(s![row, ..]).assign(&weights);
                self.num_used += 1;
            }
        }
    }
}

pub mod mesolimbic_system {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MesolimbicSystem {
        pub reward_threshold: f32,
        pub dopamine_release_rate: f32,
    }

    impl MesolimbicSystem {
        pub fn new(reward_threshold: f32, dopamine_release_rate: f32) -> Self {
            MesolimbicSystem {
                reward_threshold,
                dopamine_release_rate,
            }
        }

        pub fn process_reward(&self, reward: f32) -> f32 {
            if reward >= self.reward_threshold {
                self.dopamine_release_rate * reward
            } else {
                0.0
            }
        }
    }
}

pub mod ns_router_rs {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct NSRoutingPlan {
        pub model_config: ModelConfig,
        pub kv_cache_config: KVCacheConfig,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ModelConfig {
        pub precision: Vec<super::quantizer::QuantizationResult>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct KVCacheConfig {
        pub sparsity: f32,
        pub priority_tokens: Vec<u32>,
    }
}

pub mod quantizer {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct QuantizationResult {
        pub token_id: u32,
        pub salience_score: f32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TokenFeatures {
        pub token_id: u32,
        pub frequency: f32,
        pub sentiment_score: f32,
        pub context_relevance: f32,
        pub role: String,
    }

    pub struct SalienceQuantizer {
        threshold: f32,
    }

    impl SalienceQuantizer {
        pub fn new(threshold: f32) -> Self {
            SalienceQuantizer { threshold }
        }

        pub fn quantize_tokens(&self, tokens: Vec<TokenFeatures>, precision: &str) -> (Vec<QuantizationResult>, Vec<f32>) {
            let len = tokens.len();
            let results = tokens.into_iter()
                .map(|t| QuantizationResult {
                    token_id: t.token_id,
                    salience_score: t.context_relevance,
                })
                .filter(|r| r.salience_score > self.threshold)
                .collect();
            (results, vec![0.0; len])
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum PrecisionLevel {
        Bit2,
        Bit4,
        Bit8,
        Bit16,
    }
}

#[cfg(feature = "grpc")]
mod pb {
    pub mod sidecar_service_client {
        use tonic::transport::Channel;

        #[derive(Clone)]
        pub struct SidecarServiceClient<T> {
            inner: T,
        }

        impl SidecarServiceClient<Channel> {
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: Into<String>,
            {
                let inner = Channel::from_shared(dst.into())?.connect().await?;
                Ok(SidecarServiceClient { inner })
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tableau {
    pub data: Vec<u8>,
}

impl Default for Tableau {
    fn default() -> Self {
        Tableau { data: vec![] }
    }
}
