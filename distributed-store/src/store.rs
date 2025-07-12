// Copyright 2025 xAI
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
use rand::Rng;
use kube::{
    api::{Api, PostParams},
    Client,
    core::params::ListParams,
};
use k8s_openapi::api::core::v1 as k8s;
use crate::resource_tracker::ResourceTracker;
use zeta_vault_synergy::ZetaVaultSynergy;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Initialization error: {0}")]
    Init(String),
    #[error("Memory error: {0}")]
    Memory(String),
    #[error("Transfer error: {0}")]
    Transfer(String),
    #[error("Kubernetes error: {0}")]
    K8s(#[from] kube::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Slice {
    ptr: *mut u8,
    size: usize,
}

impl Drop for Slice {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { std::alloc::dealloc(self.ptr, std::alloc::Layout::from_size_align(self.size, 8).unwrap()) };
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SliceBuffer {
    buffer: *mut u8,
    size: usize,
}

impl SliceBuffer {
    pub fn ptr(&self) -> *mut u8 {
        self.buffer
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct DistributedObjectStore {
    local_hostname: String,
    protocol: String,
    client_buffer: Vec<u8>,
    segment_ptr: Option<Vec<u8>>,
    vault: Arc<ZetaVaultSynergy>,
    kube_client: Client,
    resource_tracker: Arc<ResourceTracker>,
}

impl DistributedObjectStore {
    pub fn new(vault: Arc<ZetaVaultSynergy>) -> Arc<Self> {
        let store = Arc::new(DistributedObjectStore {
            local_hostname: String::new(),
            protocol: String::new(),
            client_buffer: Vec::new(),
            segment_ptr: None,
            vault,
            kube_client: Client::try_default().unwrap(),
            resource_tracker: ResourceTracker::new(),
        });
        store.resource_tracker.register_instance(Arc::clone(&store));
        store
    }

    async fn provision_k8s_resources(&self, batch_size: usize) -> Result<(), StoreError> {
        let pods: Api<k8s::Pod> = Api::default_namespaced(self.kube_client.clone());
        for i in 0..batch_size {
            let pod_name = format!("store-pod-{}", i);
            let pod = k8s::Pod {
                metadata: k8s::ObjectMeta {
                    name: Some(pod_name.clone()),
                    labels: Some(std::collections::BTreeMap::from([("app".to_string(), "store".to_string())])),
                    ..Default::default()
                },
                spec: Some(k8s::PodSpec {
                    containers: vec![k8s::Container {
                        name: "store-container".to_string(),
                        image: Some("xai/distributed-store:latest".to_string()),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            };
            pods.create(&PostParams::default(), &pod).await?;
            log::info!("Provisioned pod {}", pod_name);
        }
        Ok(())
    }

    pub async fn setup(
        &mut self,
        local_hostname: String,
        metadata_server: String,
        global_segment_size: usize,
        local_buffer_size: usize,
        protocol: String,
        rdma_devices: String,
        master_server_addr: String,
    ) -> Result<(), StoreError> {
        self.protocol = protocol;
        self.local_hostname = if local_hostname.contains(":") {
            local_hostname
        } else {
            let port = rand::thread_rng().gen_range(12300..14300);
            format!("{}:{}", local_hostname, port)
        };

        self.client_buffer = vec![0; local_buffer_size];
        self.vault.store("client_buffer", self.client_buffer.clone()).await?;

        if global_segment_size > 0 {
            self.segment_ptr = Some(vec![0; global_segment_size]);
            self.vault.store("segment", self.segment_ptr.clone().unwrap()).await?;
        }
        self.provision_k8s_resources(1).await?;
        Ok(())
    }

    pub async fn init_all(&mut self, protocol: String, device_name: String, mount_segment_size: usize) -> Result<(), StoreError> {
        self.setup("localhost".to_string(), "127.0.0.1:2379".to_string(), mount_segment_size, 1024 * 1024 * 1024, protocol, device_name, "".to_string()).await
    }

    fn allocate_slices(&self, value: &[u8]) -> Result<Vec<Slice>, StoreError> {
        let mut slices = Vec::new();
        let mut offset = 0;
        const MAX_SLICE_SIZE: usize = 1024 * 1024; // 1MB
        while offset < value.len() {
            let chunk_size = std::cmp::min(value.len() - offset, MAX_SLICE_SIZE);
            let ptr = unsafe { std::alloc::alloc(std::alloc::Layout::from_size_align(chunk_size, 8).unwrap()) };
            if ptr.is_null() {
                return Err(StoreError::Memory("Allocation failed".to_string()));
            }
            unsafe { std::ptr::copy_nonoverlapping(value.as_ptr().add(offset), ptr, chunk_size); }
            slices.push(Slice { ptr, size: chunk_size });
            offset += chunk_size;
        }
        Ok(slices)
    }

    pub async fn put(&self, key: String, value: &[u8]) -> Result<(), StoreError> {
        let slices = self.allocate_slices(value)?;
        self.vault.store(key, value.to_vec()).await?;
        for slice in slices {
            unsafe { std::alloc::dealloc(slice.ptr, std::alloc::Layout::from_size_align(slice.size, 8).unwrap()); }
        }
        Ok(())
    }

    pub async fn get(&self, key: String) -> Result<Vec<u8>, StoreError> {
        self.vault.get(key).await.map_err(|e| StoreError::Transfer(e.to_string()))
    }

    pub async fn tear_down_all(&self) {
        self.vault.clear().await;
        let pods: Api<k8s::Pod> = Api::default_namespaced(self.kube_client.clone());
        let lp = ListParams::default().labels("app=store");
        if let Ok(pod_list) = pods.list(&lp).await {
            for pod in pod_list.items {
                if let Some(name) = pod.metadata.name {
                    pods.delete(&name, &Default::default()).await.ok();
                    log::info!("Deleted pod {}", name);
                }
            }
        }
    }
}

impl Drop for DistributedObjectStore {
    fn drop(&mut self) {
        self.resource_tracker.unregister_instance(Arc::downgrade(&Arc::new(self.clone())).upgrade().unwrap());
    }
}