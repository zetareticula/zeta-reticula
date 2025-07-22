// Copyright 2025 ZETA RETICULA
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

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use p2pstore::KVCache;
use crate::attention_store::AttentionStore;
use crate::agentflow::AgentFlow;
use crate::zeta_vault_synergy::ZetaVaultSynergy;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PetriEngineError {
    #[error("Invalid transition: {0}")]
    Transition(String),
    #[error("Insufficient tokens: {0}")]
    Token(String),
}

#[derive(Debug)]
pub struct PetriPlace {
    id: usize,
    tokens: f32, // Attention weight or memory salience
}

#[derive(Debug)]
pub struct PetriTransition {
    id: usize,
    pre: Vec<usize>,
    post: Vec<usize>,
    weights: HashMap<usize, f32>,
    delta: f32, // Mass gap threshold
}

impl PetriTransition {
    fn can_fire(&self, places: &HashMap<usize, PetriPlace>) -> bool {
        self.pre.iter().map(|&p| {
            places.get(&p).map_or(0.0, |place| place.tokens * self.weights.get(&p).unwrap_or(&1.0))
        }).sum::<f32>() >= self.delta
    }

    fn fire(&mut self, places: &mut HashMap<usize, PetriPlace>) {
        if self.can_fire(places) {
            for &p in &self.pre {
                if let Some(place) = places.get_mut(&p) {
                    place.tokens *= 0.5; // Decay input tokens
                }
            }
            for &p in &self.post {
                places.entry(p).or_insert(PetriPlace { id: p, tokens: 0.0 }).tokens += 1.0; // Activate output
            }
        }
    }
}

pub struct PetriEngine {
    places: Arc<RwLock<HashMap<usize, PetriPlace>>>,
    transitions: Vec<PetriTransition>,
    attention_store: Arc<AttentionStore>,
    agent_flow: Arc<AgentFlow>,
    vault: Arc<ZetaVaultSynergy>,
    mass_gap: f32,
}

impl PetriEngine {
    pub fn new(attention_store: Arc<AttentionStore>, agent_flow: Arc<AgentFlow>, vault: Arc<ZetaVaultSynergy>, mass_gap: f32) -> Arc<Self> {
        let mut places = HashMap::new();
        places.insert(0, PetriPlace { id: 0, tokens: 0.0 }); // Initial KV cache place
        Arc::new(PetriEngine {
            places: Arc::new(RwLock::new(places)),
            transitions: vec![
                PetriTransition {
                    id: 0,
                    pre: vec![0],
                    post: vec![1],
                    weights: HashMap::from([(0, 1.0)]),
                    delta: mass_gap,
                },
            ],
            attention_store,
            agent_flow,
            vault,
            mass_gap,
        })
    }

    pub async fn update_kv_cache(&self, model_id: &str, kv_cache: &[KVCache], bit_width: u8, lora_enabled: bool) {
        let mut places = self.places.write().await;
        let mut total_energy = 0.0;
        for cache in kv_cache {
            total_energy += cache.buffers.iter().map(|b| b.size_ as f32).sum::<f32>();
        }
        if total_energy >= self.mass_gap {
            places.get_mut(&0).unwrap().tokens = total_energy;
            for transition in &mut self.transitions {
                transition.fire(&mut places);
            }
            self.attention_store.store_kv_cache(model_id, kv_cache.to_vec()).await.ok();
            self.agent_flow.enqueue_task(agentflow::AgentTask::Quantization { model_id: model_id.to_string(), bit_width }, 5).await.ok();
        }
    }

    pub fn get_agent_status(&self) -> serde_json::Value {
        let places = self.places.blocking_read();
        serde_json::json!({
            "places": places.iter().map(|(id, p)| (id, p.tokens)).collect::<Vec<_>>(),
            "mass_gap": self.mass_gap,
        })
    }
}