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


use crate::{AgentFlowServer, AgentTask};
use kvquant_rs::{kv_cache::KVCache, KVQuantizer as Quantizer};
use kvquant_rs::pb::sidecar_service_client::SidecarServiceClient;
// QuantizedFeatures not available in kvquant_rs::pb
use log::{info, error};
use prost::Message;
use std::sync::Arc;
use std::thread;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct AgentFlowQuantizer {
    pub sidecar_address: String,
    pub channel: mpsc::Sender<AgentTask>,
}

impl AgentFlowQuantizer {
    pub async fn new(sidecar_address: String, server: Arc<AgentFlowServer>) -> Self {
        let (tx, rx) = mpsc::channel(100);

        thread::spawn(move || loop {
            let request = rx.recv().expect("Failed to receive request");
            info!("Received request: {:?}", request);

            match request {
                AgentTask::Quantization { model_id, bit_width } => {
                    let mut client = SidecarServiceClient::connect(sidecar_address.clone()).unwrap();

                    let token_features: Vec<KVCache> = server.attention_store.get_token_features(model_id.clone());
                    let quantized_features = server.quantizer.quantize(token_features, bit_width);

                    // Create a simple message structure since QuantizedFeatures is not available
                    let quantized_features_bytes = format!("model_id:{},bit_width:{}", model_id, bit_width).into_bytes();
                    let request = tonic::Request::new(quantized_features_bytes);
                    let response = client.store_quantized_features(request).unwrap();
                    info!("Store quantized features response: {:?}", response);
                }
                _ => {}
            }
        });

        Self {
            sidecar_address,
            channel: tx,
        }
    }

    pub async fn quantize(&self, model_id: String, bit_width: usize) {
        self.channel
            .send(AgentTask::Quantization {
                model_id,
                bit_width,
            })
            .await
            .unwrap();
    }
}

// Quantizer trait implementation removed - KVQuantizer is not a trait
