// Copyright 2025 Zeta Reticula Inc
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

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use log;
use kube::{
    api::{Api, PostParams},
    Client,
    core::params::ListParams,
    ApiResource,
};
use k8s_openapi::api::core::v1 as k8s;
use crate::memory_manager::MemoryManager;
use zeta_vault_synergy::ZetaVaultSynergy;
use agentflow_rs::AgentTask;

#[derive(Error, Debug)]
pub enum TransferEngineError {
    #[error("Initialization error: {0}")]
    Init(String),
    #[error("Memory error: {0}")]
    Memory(#[from] MemoryManagerError),
    #[error("Transfer error: {0}")]
    Transfer(String),
    #[error("Kubernetes error: {0}")]
    K8s(#[from] kube::Error),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferOpcode {
    Read,
    Write,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferRequest {
    opcode: TransferOpcode,
    source: usize,
    target_id: i64,
    target_offset: u64,
    length: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TransferStatus {
    Waiting,
    Pending,
    Invalid,
    Canceled,
    Completed,
    Timeout,
    Failed,
}

pub struct TransferEngine {
    metadata_conn_string: String,
    local_server_name: String,
    local_ip_address: String,
    rpc_port: u16,
    memory_manager: Arc<Mutex<MemoryManager>>,
    vault: Arc<ZetaVaultSynergy>,
    transfer_timeout: Duration,
    kube_client: Client,
}

impl TransferEngine {
    pub async fn new(
        metadata_conn_string: String,
        local_server_name: String,
        local_ip_address: String,
        rpc_port: u16,
        vault: Arc<ZetaVaultSynergy>,
    ) -> Result<Self, TransferEngineError> {
        let timeout = Duration::from_secs(30); // Default 30s timeout
        if let Ok(timeout_sec) = std::env::var("MC_TRANSFER_TIMEOUT").unwrap_or_default().parse::<u64>() {
            if timeout_sec >= 5 {
                timeout = Duration::from_secs(timeout_sec);
            }
        }
        let memory_manager = Arc::new(Mutex::new(MemoryManager::new()));
        let kube_client = Client::try_default().await?;

        Ok(TransferEngine {
            metadata_conn_string,
            local_server_name,
            local_ip_address,
            rpc_port,
            memory_manager,
            vault,
            transfer_timeout,
            kube_client,
        })
    }

    async fn provision_k8s_resources(&self, batch_size: usize) -> Result<(), TransferEngineError> {
        let pods: Api<k8s::Pod> = Api::default_namespaced(self.kube_client.clone());
        let services: Api<k8s::Service> = Api::default_namespaced(self.kube_client.clone());

        // Dynamically provision pods for batch processing
        for i in 0..batch_size {
            let pod_name = format!("transfer-pod-{}", i);
            let pod = k8s::Pod {
                metadata: k8s::ObjectMeta {
                    name: Some(pod_name.clone()),
                    labels: Some(BTreeMap::from([("app".to_string(), "transfer".to_string())])),
                    ..Default::default()
                },
                spec: Some(k8s::PodSpec {
                    containers: vec![k8s::Container {
                        name: "transfer-container".to_string(),
                        image: Some("xai/transfer-engine:latest".to_string()),
                        ports: Some(vec![k8s::ContainerPort {
                            container_port: self.rpc_port as i32,
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            };
            pods.create(&PostParams::default(), &pod).await?;
            log::info!("Provisioned pod {}", pod_name);
        }

        // Provision service for load balancing
        let service_name = "transfer-service";
        let service = k8s::Service {
            metadata: k8s::ObjectMeta {
                name: Some(service_name.to_string()),
                ..Default::default()
            },
            spec: Some(k8s::ServiceSpec {
                selector: Some(BTreeMap::from([("app".to_string(), "transfer".to_string())])),
                ports: Some(vec![k8s::ServicePort {
                    port: self.rpc_port as i32,
                    target_port: Some(k8s_openapi::IntOrString::Int(self.rpc_port as i32)),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        services.create(&PostParams::default(), &service).await?;
        log::info!("Provisioned service {}", service_name);

        Ok(())
    }

    pub async fn install_transport(&self, protocol: String, topology_matrix: String) -> Result<(), TransferEngineError> {
        log::info!("Installed transport {} with topology {}", protocol, topology_matrix);
        Ok(())
    }

    pub async fn uninstall_transport(&self, protocol: String) -> Result<(), TransferEngineError> {
        log::info!("Uninstalled transport {}", protocol);
        Ok(())
    }

    pub async fn register_local_memory(&self, addr: usize, length: u64, location: String) -> Result<(), TransferEngineError> {
        let mut manager = self.memory_manager.lock().unwrap();
        manager.register_memory(addr, length, location)
    }

    pub async fn unregister_local_memory(&self, addr: usize) -> Result<(), TransferEngineError> {
        let mut manager = self.memory_manager.lock().unwrap();
        manager.unregister_memory(addr)
    }

    pub async fn allocate_batch_id(&self, batch_size: usize) -> Result<i64, TransferEngineError> {
        self.provision_k8s_resources(batch_size).await?;
        let batch_id = rand::random::<i64>();
        if batch_id < 0 {
            return Err(TransferEngineError::Transfer("Batch ID allocation failed".to_string()));
        }
        Ok(batch_id)
    }

    pub async fn submit_transfer(&self, batch_id: i64, requests: Vec<TransferRequest>) -> Result<(), TransferEngineError> {
        let start = Instant::now();
        let mut tasks = Vec::new();
        for req in requests {
            let elapsed = start.elapsed();
            if elapsed > self.transfer_timeout {
                return Err(TransferEngineError::Transfer("Transfer timeout".to_string()));
            }
            let key = format!("transfer_{}_{}", batch_id, req.target_id);
            let data = vec![0u8; req.length as usize]; // Placeholder data
            self.vault.store(key, data).await.map_err(|e| TransferEngineError::Transfer(e.to_string()))?;
            tasks.push(AgentTask::Transfer(req)); // Integrate with AgentFlow
            log::info!("Transferred {} bytes to target {}", req.length, req.target_id);
        }
        // Submit tasks to AgentFlow for concurrent execution
        // (Assuming AgentFlow integration is available)
        Ok(())
    }

    pub async fn get_transfer_status(&self, batch_id: i64, task_id: usize) -> Result<(TransferStatus, u64), TransferEngineError> {
        // Mock status check with frame-dependent batch gets
        let pods: Api<k8s::Pod> = Api::default_namespaced(self.kube_client.clone());
        let lp = ListParams::default().labels("app=transfer");
        let pod_list = pods.list(&lp).await?;
        let status = if rand::random::<f64>() > 0.9 || pod_list.items.len() > task_id {
            TransferStatus::Completed
        } else {
            TransferStatus::Pending
        };
        let transferred_bytes = if status == TransferStatus::Completed { 1024 } else { 0 };
        Ok((status, transferred_bytes))
    }

    pub async fn free_batch_id(&self, batch_id: i64) -> Result<(), TransferEngineError> {
        // Cleanup k8s resources
        let pods: Api<k8s::Pod> = Api::default_namespaced(self.kube_client.clone());
        let lp = ListParams::default().labels("app=transfer");
        let pod_list = pods.list(&lp).await?;
        for pod in pod_list.items {
            if let Some(name) = pod.metadata.name {
                pods.delete(&name, &Default::default()).await?;
                log::info!("Deleted pod {}", name);
            }
        }
        let services: Api<k8s::Service> = Api::default_namespaced(self.kube_client.clone());
        services.delete("transfer-service", &Default::default()).await?;
        log::info!("Deleted service transfer-service");
        Ok(())
    }

    pub async fn open_segment(&self, segment_name: String, use_cache: bool) -> Result<i64, TransferEngineError> {
        let segment_id = rand::random::<i64>();
        if segment_id < 0 {
            return Err(TransferEngineError::Transfer("Segment open failed".to_string()));
        }
        Ok(segment_id)
    }

    pub async fn close_segment(&self, segment_id: i64) -> Result<(), TransferEngineError> {
        log::info!("Closed segment {}", segment_id);
        Ok(())
    }
}