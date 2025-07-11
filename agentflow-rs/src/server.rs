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
use tokio::sync::mpsc;
use std::sync::Arc;
use dashmap::DashMap;
use crate::client::Client;
use kvquant-rs::{initialize_kv_cache, KVQuantConfig};
use crate::config::AgentFlowConfig;
use futures::future::join_all;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use futures::future::join_all;
use log;

#! [allow(unused_imports)]
// This module defines the AgentFlowServer which manages multiple clients and their configurations.
// The AgentFlowServer is responsible for managing the clients, initializing them, and handling their configurations.
// AgentFlowServer is the main server structure for managing clients and their configurations
// It initializes clients with their respective configurations and manages their lifecycle.

pub mod client;
pub mod config;



#[derive(Serialize, Deserialize)]
pub struct AgentFlowServer {
    clients: DashMap<usize, Arc<Client>>,
    config: AgentFlowConfig,
}

impl AgentFlowServer {
    pub fn new(config: AgentFlowConfig) -> Self {
        let clients = DashMap::new();
        let kv_cache_config = KVQuantConfig {
            block_size: 100,
            spot_capacity: 10,
        };
        let kv_cache = Arc::new(initialize_kv_cache(kv_cache_config));

        for id in 0..config.num_clients {
            let client = Client::new(id, 768, 100, kv_cache.clone(), format!("client_{}_data.bin", id));
            clients.insert(id, Arc::new(client));
        }

        AgentFlowServer { clients, config }
    }

    pub async fn initialize(&self) {
        let futures: Vec<_> = self.clients.iter()
            .map(|entry| {
                let mut client = entry.value().clone();
                async move {
                    client.initialize().await;
                }
            })
            .collect();
        futures::join_all(futures).await;
    }
}

    pub async fn run(&self) {
        let mut tasks = FuturesUnordered::new();
        for entry in self.clients.iter() {
            let client = entry.value().clone();
            tasks.push(tokio::spawn(async move {
                client.run().await;
            }));
        }
        while let Some(_) = tasks.next().await {}
    }

    pub fn get_client(&self, id: usize) -> Option<Arc<Client>> {
        self.clients.get(&id).map(|c| c.clone())
    }
}

pub fn initialize_agent_flow_server(config: AgentFlowConfig) -> AgentFlowServer {
    log::info!("Initializing AgentFlowServer with {} clients", config.num_clients);
    AgentFlowServer::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AgentFlowConfig;

    #[tokio::test]
    async fn test_agent_flow_server_initialization() {
        let config = AgentFlowConfig {
            num_clients: 5,
            client_config: Default::default(),
        };
        let server = initialize_agent_flow_server(config);
        server.initialize().await;
        assert_eq!(server.clients.len(), 5);
    }

    #[tokio::test]
    async fn test_agent_flow_server_run() {
        let config = AgentFlowConfig {
            num_clients: 3,
            client_config: Default::default(),
        };
        let server = initialize_agent_flow_server(config);
        server.initialize().await;
        server.run().await;
    }
}


#[derive(Serialize, Deserialize)]
pub struct AgentFlowConfig {
    pub num_clients: usize,
    pub client_config: ClientConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ClientConfig {
    pub input_size: usize,
    pub output_size: usize,
    pub kv_cache_config: KVQuantConfig,
}

impl Default for AgentFlowConfig {
    fn default() -> Self {
        AgentFlowConfig {
            num_clients: 1,
            client_config: ClientConfig {
                input_size: 768,
                output_size: 100,
                kv_cache_config: KVQuantConfig {
                    block_size: 100,
                    spot_capacity: 10,
                },
            },
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            input_size: 768,
            output_size: 100,
            kv_cache_config: KVQuantConfig {
                block_size: 100,
                spot_capacity: 10,
            },
        }
    }
}


#[derive(Serialize, Deserialize)]
