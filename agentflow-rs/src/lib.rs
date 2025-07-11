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


use serde::{Serialize, Deserialize, Debug};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentFlowConfig {
    pub num_clients: usize,
    pub privacy_epsilon: f32,
}

pub fn initialize_agent_flow(config: AgentFlowConfig) -> server::AgentFlowServer {
    log::info!("Initializing agentflow-rs with {} clients", config.num_clients);
    server::AgentFlowServer::new(config)
}
