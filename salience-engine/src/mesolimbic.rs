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


use serde::{Serialize, Deserialize};
use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::sync::Arc;
use crate::role_inference::{RoleInfererImpl, RoleInferenceResult, TokenFeatures, SalienceResult};
use crate::role_inferer::RoleInferer;

/// Mesolimbic configuration
#[derive(Serialize, Deserialize)]
pub struct MesolimbicConfig {
    pub outer_loop_iterations: usize,
    pub inner_loop_iterations: usize,
}

//
impl MesolimbicConfig {
    pub fn new(outer_loop_iterations: usize, inner_loop_iterations: usize) -> Self {
        Self {
            outer_loop_iterations,
            inner_loop_iterations,
        }
    }
}

pub struct MesolimbicSystem {
    role_inferer: Arc<RoleInfererImpl>,
}

impl MesolimbicSystem {
    pub fn with_config(config: MesolimbicConfig) -> Self {
        MesolimbicSystem {
            role_inferer: Arc::new(RoleInfererImpl::new(
                config.outer_loop_iterations,
                config.inner_loop_iterations,
            )),
        }
    }

    pub fn compute_salience(&self, features: Vec<TokenFeatures>, theory_key: &str) -> Vec<SalienceResult> {
        let role_results = self.role_inferer.infer_roles(features.clone(), theory_key);

        features
            .into_iter()
            .zip(role_results)
            .map(|(feature, role_result)| {
                let mut rng = rand::thread_rng();

                let weights = [0.3, 0.25, 0.2, 0.1, 0.05, 0.15];
                let inputs = [
                    feature.frequency,
                    feature.sentiment_score.abs(),
                    feature.context_relevance,
                    feature.context_relevance * 0.1,
                    rng.gen_range(-0.05..0.05),
                    feature.sentiment_score.abs() * 0.15,
                ];

                let mut salience_score: f32 = weights
                    .iter()
                    .zip(inputs.iter())
                    .map(|(w, v)| w * v)
                    .sum();

                let role_modulation = match role_result.role.as_str() {
                    "negation" => 0.2,
                    "subject" => 0.15,
                    "object" => 0.1,
                    _ => 0.0,
                };

                salience_score += role_modulation;
                salience_score = salience_score.clamp(0.0, 1.0);

                SalienceResult {
                    token_id: feature.token_id,
                    salience_score,
                    role: role_result.role,
                    role_confidence: role_result.confidence,
                }
            })
            .collect()
    }
}


trait Normalize {
    fn normalize(&mut self);
}

impl Normalize for Vec<f32> {
    fn normalize(&mut self) {
        let sum: f32 = self.iter().sum();
        if sum > 0.0 {
            for val in self.iter_mut() {
                *val /= sum;
            }
        }
    }
}