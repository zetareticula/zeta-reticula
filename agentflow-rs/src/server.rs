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