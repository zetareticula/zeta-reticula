#!/bin/bash
set -e

# Colors for output
cyan="\033[0;36m"
red="\033[0;31m"
green="\033[0;32m"
yellow="\033[0;33m"
reset="\033[0m"

function log() {
    echo -e "${cyan}==> $1${reset}"
}

function error() {
    echo -e "${red}==> $1${reset}"
    exit 1
}

function success() {
    echo -e "${green}==> $1${reset}"
}

function warning() {
    echo -e "${yellow}Warning: $1${reset}"
}

function create_workspace_config() {
    log "Creating workspace configuration..."
    
    # Create root Cargo.toml with workspace config
    cat > "Cargo.toml" << EOF
[workspace]
members = [
    "api",
    "salience-engine",
    "llm-rs",
    "ns-router-rs",
    "kvquant-rs",
    "quantize-cli",
    "agentflow-rs",
]
resolver = "2"
EOF
}

function fix_agentflow_cargo() {
    log "Fixing agentflow-rs/Cargo.toml..."
    
    # Create package-specific Cargo.toml
    cat > "agentflow-rs/Cargo.toml" << EOF
[package]
name = "agentflow-rs"
version = "0.1.0"
edition = "2021"

[dependencies]
salience-engine = { path = "../salience-engine", optional = true }
ns-router-rs = { path = "../ns-router-rs", optional = true }
kvquant-rs = { path = "../kvquant-rs", optional = true }
llm-rs = { path = "../llm-rs", optional = true }
serde = { version = "1.0", features = ["derive"], optional = true }
serde_json = { version = "1.0", optional = true }
rayon = { version = "1.8", optional = true }
tokio = { version = "1.0", features = ["full"], optional = true }
dashmap = { version = "5.5", optional = true }
log = { version = "0.4", optional = true }
rand = { version = "0.8", optional = true }
ndarray = { version = "0.15", optional = true }
half = { version = "2.2", optional = true }
openblas-src = { version = "0.10.12" }
rustc-hash = { version = "1.1", optional = true }
argmin = { version = "0.8", optional = true }
pyo3 = { version = "0.19", optional = true, features = ["extension-module"] }
mlua = { version = "0.9", optional = true }
validator = { version = "0.16", features = ["derive"], optional = true }
wasm-bindgen = { version = "0.2.87", optional = true }
wasm-bindgen-futures = { version = "0.4.37", optional = true }
js-sys = { version = "0.3.64", optional = true }
uuid = { version = "1.2", features = ["v4"], optional = true }
chrono = { version = "0.4", features = ["serde"], optional = true }
stripe = { version = "0.0.5", optional = true }
reqwest = { version = "0.11", features = ["json", "blocking"], optional = true }
serde_qs = { version = "0.9", optional = true }
prost = { version = "0.12", optional = true }
tonic = { version = "0.10", optional = true }
thiserror = { version = "1.0", optional = true }
actix-web = { version = "4.3", optional = true }
actix-multipart = { version = "0.6", optional = true }
futures = { version = "0.3", optional = true }
sled = { version = "0.34", optional = true }
crossbeam = { version = "0.8", optional = true }
bumpalo = { version = "3.19.0", optional = true }
rand_distr = { version = "0.4", optional = true }

[features]
default = ["server"]
server = ["tokio", "actix-web", "actix-multipart", "sled", "crossbeam", "dashmap", "ndarray", "half", "openblas-src", "argmin"]
wasm = ["wasm-bindgen", "wasm-bindgen-futures", "llm-rs/wasm", "salience-engine/wasm", "ndarray", "half"]
python = ["pyo3"]
lua = ["mlua"]

[lib]
crate-type = ["cdylib", "rlib"]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
EOF
}

function fix_lib_rs() {
    log "Fixing agentflow-rs/src/lib.rs..."
    
    # Create fixed lib.rs
    cat > "agentflow-rs/src/lib.rs" << EOF
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
EOF
}

function verify_changes() {
    log "Verifying changes..."
    
    # Check if Cargo.toml exists and is valid
    if [ ! -f "Cargo.toml" ]; then
        error "Failed to create root Cargo.toml"
    fi
    
    # Check if agentflow-rs/Cargo.toml exists and is valid
    if [ ! -f "agentflow-rs/Cargo.toml" ]; then
        error "Failed to create agentflow-rs/Cargo.toml"
    fi
    
    # Check if lib.rs exists and is valid
    if [ ! -f "agentflow-rs/src/lib.rs" ]; then
        error "Failed to create lib.rs"
    fi
    
    # Try to build the project
    log "Attempting to build the project..."
    pushd agentflow-rs > /dev/null
    cargo build --features server
    popd > /dev/null
    
    success "All changes verified successfully!"
}

function main() {
    log "Starting agentflow-rs configuration fix..."
    
    # Create workspace config
    create_workspace_config
    
    # Fix agentflow-rs config
    fix_agentflow_cargo
    
    # Fix lib.rs
    fix_lib_rs
    
    # Verify changes
    verify_changes
}

main "$@"
