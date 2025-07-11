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
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use crate::server::AgentFlowServer;
use rayon::prelude::*;
use futures::future::join_all;
use dashmap::DashMap;


// This module defines the DistributedIO struct which provides parallel read functionality
// for distributed clients in the AgentFlowServer.

#[derive(Serialize, Deserialize)]
pub struct DistributedIO;

impl DistributedIO {
    pub async fn parallel_read(&self, server: &AgentFlowServer, chunk_size: usize) -> Vec<u8> {
        let futures: Vec<_> = server.clients.par_iter()
            .map(|entry| {
                let client = entry.value();
                let path = client.local_data.clone();
                async move {
                    let file = File::open(&path).await.unwrap();
                    let mut reader = BufReader::new(file);
                    let mut buffer = vec![0u8; chunk_size];
                    reader.read_exact(&mut buffer).await.unwrap();
                    buffer
                }
            })
            .collect();

        let results = futures::join_all(futures).await;
        results.into_iter().flatten().collect()
    }
}