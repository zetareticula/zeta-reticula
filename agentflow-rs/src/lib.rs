use serde::{Serialize, Deserialize};
use log;
use bumpalo::Bump;
use rayon::prelude::*;
use std::sync::{Arc, RwLock};
use dashmap::DashMap;
use rand_distr::{Distribution, Normal};
use crate::client::AgentFlowClient;
use crate::server::AgentFlowServer;
use crate::anss::AgentNetworkSalienceSystem;
use crate::cache::CacheManager;
use crate::io::IOManager;
use crate::role::RoleInferer;
// use crate::privacy::DifferentialPrivacyManager;
// use crate::privacy::DifferentialPrivacyManager;
// use crate::privacy::DifferentialPrivacyManager;



pub mod quantizer;
pub mod client;
pub mod server;
pub mod anss;
pub mod cache;
pub mod io;
pub mod role;
pub mod privacy;
pub mod block;
pub mod tableaux;
pub mod mesolimbic;
pub mod spot;
pub mod role_inference;


#[derive(Serialize, Deserialize)]
pub struct AgentFlowConfig {
    pub num_clients: usize,
    pub privacy_epsilon: f32,  // Differential privacy parameter
}

pub fn initialize_agent_flow(config: AgentFlowConfig) -> server::AgentFlowServer {
    log::info!("Initializing agentflow-rs with {} clients", config.num_clients);
    server::AgentFlowServer::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::AgentFlowServer;
    use crate::client::AgentFlowClient;

    #[tokio::test]
    async fn test_agent_flow_initialization() {
        let config = AgentFlowConfig {
            num_clients: 5,
            privacy_epsilon: 0.1,
        };
        let server = initialize_agent_flow(config);
        server.initialize().await;
        assert_eq!(server.clients.len(), 5);
    }

    #[tokio::test]
    async fn test_agent_flow_client_interaction() {
        let config = AgentFlowConfig {
            num_clients: 3,
            privacy_epsilon: 0.1,
        };
        let server = initialize_agent_flow(config);
        server.initialize().await;

        let client = AgentFlowClient::new(1, &server);
        client.run().await;
    }
}