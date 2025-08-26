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

//! Master service for Zeta Reticula's distributed AI system

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::time;
use uuid::Uuid;

use thiserror::Error;
use tonic::{transport::Server, Request, Response, Status};

pub mod proto {
    tonic::include_proto!("master");
}

use proto::{
    master_service_server::{MasterService as MasterServiceTrait, MasterServiceServer},
    *,
};

#[derive(Error, Debug)]
pub enum MasterServiceError {
    #[error("Service error: {0}")]
    ServiceError(String),
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    #[error("RPC error: {0}")]
    RpcError(#[from] Status),
    #[error("Transport error: {0}")]
    TransportError(#[from] tonic::transport::Error),
}

#[derive(Debug, Clone)]
struct NodeInfo {
    id: String,
    last_seen: SystemTime,
    metadata: HashMap<String, String>,
}

/// Main master service implementation
#[derive(Clone)]
pub struct MasterService {
    nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl Default for MasterService {
    fn default() -> Self {
        Self::new()
    }
}

impl MasterService {
    /// Create a new instance of the master service
    pub fn new() -> Self {
        MasterService {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: None,
        }
    }

    /// Register a node with the master service
    pub fn register_node(&self, id: &str, metadata: HashMap<String, String>) -> Result<(), MasterServiceError> {
        let mut nodes = self.nodes.write().map_err(|e| {
            MasterServiceError::ServiceError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        let node = NodeInfo {
            id: id.to_string(),
            last_seen: SystemTime::now(),
            metadata,
        };
        
        nodes.insert(id.to_string(), node);
        Ok(())
    }

    /// Remove a node from the master service
    pub fn remove_node(&self, id: &str) -> Result<(), MasterServiceError> {
        let mut nodes = self.nodes.write().map_err(|e| {
            MasterServiceError::ServiceError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        nodes.remove(id);
        Ok(())
    }

    /// Get all registered nodes
    pub fn get_nodes(&self) -> Result<Vec<NodeInfo>, MasterServiceError> {
        let nodes = self.nodes.read().map_err(|e| {
            MasterServiceError::ServiceError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        Ok(nodes.values().cloned().collect())
    }

    /// Clean up stale nodes that haven't sent a heartbeat in the specified duration
    pub async fn cleanup_stale_nodes(&self, max_age_seconds: u64) -> Result<usize, MasterServiceError> {
        let mut nodes = self.nodes.write().map_err(|e| {
            MasterServiceError::ServiceError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        let now = SystemTime::now();
        let max_age = Duration::from_secs(max_age_seconds);
        let initial_count = nodes.len();
        
        nodes.retain(|_, node| {
            now.duration_since(node.last_seen).map_or(false, |age| age <= max_age)
        });
        
        Ok(initial_count - nodes.len())
    }

    /// Start the master service server
    pub async fn start_server(
        self,
        addr: std::net::SocketAddr,
    ) -> Result<(), MasterServiceError> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        
        // Create the gRPC server
        let svc = MasterServiceServer::new(self.clone());
        
        // Start the cleanup task
        let cleanup_interval = Duration::from_secs(30);
        let mut interval = time::interval(cleanup_interval);
        
        let service_clone = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = service_clone.cleanup_stale_nodes(60).await {
                            log::error!("Failed to clean up stale nodes: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        log::info!("Shutting down cleanup task");
                        break;
                    }
                }
            }
        });
        
        // Start the server
        log::info!("Starting master service on {}", addr);
        Server::builder()
            .add_service(svc)
            .serve_with_shutdown(addr, async move {
                let _ = shutdown_rx.recv().await;
            })
            .await?;
        
        Ok(())
    }
}

#[tonic::async_trait]
impl MasterServiceTrait for MasterService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        let node_id = req.node_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        
        self.register_node(&node_id, req.metadata)
            .map_err(|e| Status::internal(e.to_string()))?;
        
        Ok(Response::new(RegisterResponse { node_id }))
    }
    
    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Status> {
        let req = request.into_inner();
        let node_id = req.node_id;
        
        let mut nodes = self.nodes.write().map_err(|_| {
            Status::internal("Failed to acquire write lock")
        })?;
        
        if let Some(node) = nodes.get_mut(&node_id) {
            node.last_seen = SystemTime::now();
            Ok(Response::new(HeartbeatResponse { success: true }))
        } else {
            Err(Status::not_found(format!("Node {} not found", node_id)))
        }
    }
    
    async fn get_nodes(
        &self,
        _request: Request<GetNodesRequest>,
    ) -> Result<Response<GetNodesResponse>, Status> {
        let nodes = self.nodes.read().map_err(|_| {
            Status::internal("Failed to acquire read lock")
        })?;
        
        let nodes_proto = nodes.values()
            .map(|node| NodeInfo {
                id: node.id.clone(),
                last_seen: node.last_seen.elapsed()
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(-1),
                metadata: node.metadata.clone(),
            })
            .collect();
        
        Ok(Response::new(GetNodesResponse { nodes: nodes_proto }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[tokio::test]
    async fn test_register_and_get_nodes() {
        let service = MasterService::new();
        
        // Register a node
        let mut metadata = HashMap::new();
        metadata.insert("role".to_string(), "worker".to_string());
        service.register_node("test-node", metadata).unwrap();
        
        // Get nodes and verify
        let nodes = service.get_nodes().unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "test-node");
        assert_eq!(nodes[0].metadata.get("role").unwrap(), "worker");
    }
    
    #[tokio::test]
    async fn test_cleanup_stale_nodes() {
        let service = MasterService::new();
        
        // Register a node
        service.register_node("stale-node", HashMap::new()).unwrap();
        
        // Force the node to be stale by setting last_seen to the past
        {
            let mut nodes = service.nodes.write().unwrap();
            if let Some(node) = nodes.get_mut("stale-node") {
                node.last_seen = SystemTime::now() - Duration::from_secs(120);
            }
        }
        
        // Clean up nodes older than 60 seconds
        let removed = service.cleanup_stale_nodes(60).await.unwrap();
        assert_eq!(removed, 1);
        
        // Verify the node was removed
        let nodes = service.get_nodes().unwrap();
        assert!(nodes.is_empty());
    }
}
