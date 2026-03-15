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
use rand_distr::{Distribution, Normal};
use rand::thread_rng;
use rand::Rng;
use serde_json::json;
use serde_json::Value;
use serde_json::from_str;
use serde_json::to_string;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicI32;
#[cfg(feature = "server")]
use dashmap::DashMap;
#[cfg(feature = "server")]
use dashmap::mapref::entry::Entry;
use salience_engine::tableaux::YoungTableau;
use salience_engine::role_inference::{RoleInferer, RoleInferenceResult, RoleTheory, SalienceResult};
use salience_engine::mesolimbic::MesolimbicSystem;
use kvquant_rs::{QuantizationResult, PrecisionLevel, SpotManager, DataBlock, KVQuantConfig, KVQuantizer};
use kvquant_rs::spot::BlockState;

// Represents a privacy guard that adds noise to data for differential privacy
// This is used to protect sensitive data while allowing for analysis
// and quantization of the data.

#[derive(Serialize, Deserialize, Clone)]
pub struct PrivacyGuardConfig {
    pub epsilon: f32, // Privacy budget
}

impl PrivacyGuardConfig {
    pub fn new(epsilon: f32) -> Self {
        PrivacyGuardConfig { epsilon }
    }
}

#[cfg(feature = "server")]
pub struct PrivacyGuardManager {
    guards: DashMap<usize, PrivacyGuard>,
}

#[cfg(feature = "server")]
impl PrivacyGuardManager {
    pub fn new() -> Self {
        PrivacyGuardManager {
            guards: DashMap::new(),
        }
    }

    pub fn add_guard(&self, id: usize, epsilon: f32) {
        let guard = PrivacyGuard::new(epsilon);
        self.guards.insert(id, guard);
    }

    pub fn get_guard(&self, id: usize) -> Option<PrivacyGuard> {
        self.guards.get(&id).map(|g| g.clone())
    }

    pub fn apply_guard(&self, id: usize, data: &mut [f32]) {
        if let Some(guard) = self.get_guard(id) {
            guard.add_noise(data);
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PrivacyGuard {
    epsilon: f32,
}

impl PrivacyGuard {
    pub fn new(epsilon: f32) -> Self {
        PrivacyGuard { epsilon }
    }

    pub fn add_noise(&self, data: &mut [f32]) {
        let normal = Normal::new(0.0, self.epsilon as f64).unwrap();
        let mut rng = rand::thread_rng();
        for val in data.iter_mut() {
            *val += normal.sample(&mut rng) as f32;
        }
    }

    pub fn serialize(&self) -> String {
        to_string(self).unwrap()
    }

    pub fn deserialize(data: &str) -> Self {
        from_str(data).unwrap()
    }
}

