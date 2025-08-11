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

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllocatedBufferDescriptor {
    pub buffer_address_: u64,
    pub size_: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KVCache {
    pub buffers: Vec<AllocatedBufferDescriptor>,
    pub positional_encoding: Option<Vec<i32>>,
}

impl KVCache {
    pub fn new(buffers: Vec<AllocatedBufferDescriptor>) -> Self {
        KVCache {
            buffers,
            positional_encoding: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Segment {
    pub id: String,
    pub name: String,
    pub client_id: String,
}

pub trait TransferEngine {
    async fn async_load(&self, cache: &KVCache, hbm: &mut Vec<KVCache>) -> Result<(), TransferEngineError>;
    async fn async_save(&self, cache: Vec<KVCache>) -> Result<(), TransferEngineError>;
}

#[derive(Debug)]
pub enum TransferEngineError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Buffer overflow: {0}")]
    Overflow(String),
}


