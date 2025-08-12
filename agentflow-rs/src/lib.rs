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
use log;
#[cfg(feature = "server")]
use bumpalo::Bump;
#[cfg(feature = "server")]
use rayon::prelude::*;
#[cfg(feature = "server")]
use std::sync::{Arc, RwLock};
#[cfg(feature = "server")]
use dashmap::DashMap;
#[cfg(feature = "server")]
use rand_distr::{Distribution, Normal};
#[cfg(feature = "server")]
use crate::client::AgentFlowClient;
#[cfg(feature = "server")]
use crate::server::AgentFlowServer;
#[cfg(feature = "server")]
use crate::anss::AgentNetworkSalienceSystem;
#[cfg(feature = "server")]
use crate::cache::CacheManager;
#[cfg(feature = "server")]
use crate::io::IOManager;
#[cfg(feature = "server")]
use crate::role::RoleInferer;

pub mod quantizer;
#[cfg(feature = "server")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "server")]
pub mod anss;
#[cfg(feature = "server")]
pub mod cache;
#[cfg(feature = "server")]
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

#[cfg(feature = "server")]
pub fn initialize_agent_flow(config: AgentFlowConfig) -> server::AgentFlowServer {
    log::info!("Initializing agentflow-rs with {} clients", config.num_clients);
    server::AgentFlowServer::new(config)
}
