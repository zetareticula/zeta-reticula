use serde::{Serialize, Deserialize};
use rand::distributions::{Distribution, Normal};
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
use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use crate::tableaux::YoungTableau;
use crate::role_inference::{RoleInferer, RoleInferenceResult};
use crate::mesolimbic::{MesolimbicSystem, SalienceResult};
use crate::role_inference::RoleTheory;
use crate::quantizer::{QuantizationResult, PrecisionLevel};
use crate::spot::SpotManager;
use crate::block::{DataBlock, BlockState};
use crate::quantizer::KVQuantConfig;
use crate::quantizer::KVQuantizer;

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





#[derive(Serialize, Deserialize)]
pub struct PrivacyGuard {
    epsilon: f32,
}

impl PrivacyGuard {
    pub fn new(epsilon: f32) -> Self {
        PrivacyGuard { epsilon }
    }

    pub fn add_noise(&self, data: &mut [f32]) {
        let normal = Normal::new(0.0, 1.0 / self.epsilon as f64);
        let mut rng = rand::thread_rng();
        for val in data.iter_mut() {
            *val += normal.sample(&mut rng) as f32;
        }
    }
}