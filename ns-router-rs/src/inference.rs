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

use shared::QuantizationResult;
use shared::PrecisionLevel;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use zeta_vault::ZetaVault;
use zeta_vault::VaultConfig;
use attention_store::AttentionStore;
use quantize::Quantizer;
use kvquant_rs::KVQuantizer;
use kvquant_rs::KVQuantService;






// Inference struct 
#[derive(Debug, Serialize, Deserialize)]
pub struct Inference {
    pub vault: Arc<ZetaVault>,
    pub attention_store: Arc<AttentionStore>,
    pub quantizer: Arc<Quantizer>,
}

impl Inference {
    pub fn new(vault: Arc<ZetaVault>, attention_store: Arc<AttentionStore>, quantizer: Arc<Quantizer>) -> Self {
        Inference { vault, attention_store, quantizer }
    }
}

pub fn inference(vault: Arc<ZetaVault>, attention_store: Arc<AttentionStore>, quantizer: Arc<Quantizer>) -> Inference {
    Inference::new(vault, attention_store, quantizer)
}

