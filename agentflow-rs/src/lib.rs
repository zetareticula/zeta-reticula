use serde::{Serialize, Deserialize};
use log;

pub mod client;
pub mod server;
pub mod anss;
pub mod cache;
pub mod io;
pub mod role;
pub mod privacy;

#[derive(Serialize, Deserialize)]
pub struct AgentFlowConfig {
    pub num_clients: usize,
    pub privacy_epsilon: f32,  // Differential privacy parameter
}

pub fn initialize_agent_flow(config: AgentFlowConfig) -> server::AgentFlowServer {
    log::info!("Initializing agentflow-rs with {} clients", config.num_clients);
    server::AgentFlowServer::new(config)
}