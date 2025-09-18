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


// MesolimbicSystem implements a two-loop stochastic foraging search
// within the context of the salience-engine and its working in collaboration
// with the llm-rs, zeta-vault-synergy, and the zeta-reticula crate in general.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use llm_rs;
use salience_engine::mesolimbic::MesolimbicSystem as SalienceMesolimbicSystem;
use salience_engine::role_inference::SalienceResult;
use salience_engine::tableaux::YoungTableau;
use zeta_vault_synergy;

#[derive(Serialize, Deserialize)]



pub struct AgentFlowMesolimbicSystem {
    // Outer loop parameters are of usize
    pub outer_loop_iterations: usize,
    
    pub outer_loop_threshold: f64,
    // Inner loop parameters
    pub inner_loop_iterations: usize,
    pub inner_loop_threshold: f64,
    // Salience engine parameters
    pub salience_engine: Arc<SalienceMesolimbicSystem>,
    // LLM parameters
    pub llm: Arc<YoungTableau>,
    // ZetaVaultSynergy parameters
    pub zeta_vault_synergy: Arc<zeta_vault_synergy::ZetaVaultSynergy>,
}

impl AgentFlowMesolimbicSystem {
    // Create a new MesolimbicSystem with default parameters
    pub fn new() -> Self {
        Self {
            outer_loop_iterations: 100,
            outer_loop_threshold: 0.5,
            inner_loop_iterations: 10,
            inner_loop_threshold: 0.5,
            salience_engine: Arc::new(SalienceMesolimbicSystem::new()),
            llm: Arc::new(YoungTableau::new()),
            zeta_vault_synergy: Arc::new(zeta_vault_synergy::ZetaVaultSynergy::new()),
        }
    }

    // Compute salience scores for the given tokens
    pub fn compute_salience(&self, tokens: &[u32]) -> SalienceResult {
        // Compute salience scores for each token
        let salience_results = self.salience_engine.compute_salience(tokens);
        // Compute foraging probabilities for each token
        let mut foraging_probabilities = HashMap::new();
        for (token, salience_result) in salience_results.iter() {
            let mut foraging_probability = 0.0;
            for _ in 0..self.inner_loop_iterations {
                // Generate a question about the token
                let question = self.llm.generate_question(token);
                // Ask the question to the ZetaVaultSynergy
                let response = self.zeta_vault_synergy.ask_question(&question);
                // Compute the foraging probability for the token
                foraging_probability += response.foraging_probability;
            }
            foraging_probabilities.insert(token.clone(), foraging_probability);
        }
        // Return the salience results with the foraging probabilities
        SalienceResult {
            salience_results,
            foraging_probabilities,
        }
    }
}
