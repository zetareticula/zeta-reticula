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
use std::time::Duration;
use tokio::sync::RwLock;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use log;
use uuid::Uuid;
use p2pstore::{TransferEngine, TransferRequest, TransferStatus, TransferFuture, TransferEngineError, AllocatedBufferDescriptor};
use zeta_vault_synergy::ZetaVaultSynergy;
use etcd_client::Client as EtcdClient;
use tonic::{transport::Channel, Request};
use crate::ping_task;

include!(concat!(env!("OUT_DIR"), "/mooncake.rs")); // Generated protobuf definitions

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Initialization error: {0}")]
    Init(String),
    #[error("Transfer error: {0}")]
    Transfer(String),
    #[error("Etcd error: {0}")]
    Etcd(#[from] etcd_client::Error),
    #[error("GRPC error: {0}")]
    Grpc(#[from] tonic::Status),
    #[error("Transfer engine error: {0}")]
    TransferEngine(#[from] TransferEngineError),
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
pub struct Segment {
    id: String,
    hostname: String,
    base: usize,
    size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplicateConfig {
    replica_num: usize,
    preferred_segment: String,
}

pub struct Client {
    local_hostname: String,
    metadata_connstring: String,
    client_id: String,
    transfer_engine: Arc<TransferEngine>,
    mounted_segments: Mutex<Vec<Segment>>,
    vault: Arc<ZetaVaultSynergy>,
    ping_running: std::sync::atomic::AtomicBool,
    master_client: MasterClient,
}

pub struct MasterClient {
    channel: Channel,
    client: mooncake::master_service_client::MasterServiceClient<Channel>,
}

impl MasterClient {
    pub async fn connect(address: String) -> Result<Self, ClientError> {
        let channel = Channel::from_shared(address)?.connect().await?;
        let client = mooncake::master_service_client::MasterServiceClient::new(channel.clone());
        Ok(MasterClient { channel, client })
    }

    pub async fn reconnect(&mut self, address: String) -> Result<(), ClientError> {
        self.channel = Channel::from_shared(address)?.connect().await?;
        self.client = mooncake::master_service_client::MasterServiceClient::new(self.channel.clone());
        Ok(())
    }

    pub async fn ping(&mut self, client_id: String) -> Result<PingResponse, ClientError> {
        let request = Request::new(PingRequest { client_id });
        let response = self.client.ping(request).await?.into_inner();
        Ok(response)
    }

    pub async fn get_replica_list(&mut self, key: String) -> Result<Vec<ReplicaDescriptor>, ClientError> {
        let request = Request::new(GetReplicaListRequest { key });
        let response = self.client.get_replica_list(request).await?.into_inner();
        Ok(response.replica_list)
    }

    pub async fn batch_get_replica_list(&mut self, keys: Vec<String>) -> Result<HashMap<String, Vec<ReplicaDescriptor>>, ClientError> {
        let request = Request::new(BatchGetReplicaListRequest { keys });
        let response = self.client.batch_get_replica_list(request).await?.into_inner();
        Ok(response.batch_replica_list)
    }

    pub async fn put_start(&mut self, key: String, lengths: Vec<usize>, size: usize, config: ReplicateConfig) -> Result<(bool, Vec<ReplicaDescriptor>), ClientError> {
        let request = Request::new(PutStartRequest {
            key,
            slice_lengths: lengths,
            total_size: size as i64,
            replica_num: config.replica_num as i32,
            preferred_segment: config.preferred_segment,
        });
        let response = self.client.put_start(request).await?.into_inner();
        Ok((response.error_code == 0, response.replica_list))
    }

    pub async fn put_end(&mut self, key: String) -> Result<bool, ClientError> {
        let request = Request::new(PutEndRequest { key });
        let response = self.client.put_end(request).await?.into_inner();
        Ok(response.error_code == 0)
    }

    pub async fn batch_put_start(&mut self, keys: Vec<String>, lengths: HashMap<String, usize>, slice_lengths: HashMap<String, Vec<usize>>, config: ReplicateConfig) -> Result<(bool, HashMap<String, Vec<ReplicaDescriptor>>), ClientError> {
        let request = Request::new(BatchPutStartRequest {
            keys,
            value_lengths: lengths.into_iter().map(|(k, v)| (k, v as i64)).collect(),
            slice_lengths: slice_lengths.into_iter().map(|(k, v)| (k, v)).collect(),
            replica_num: config.replica_num as i32,
            preferred_segment: config.preferred_segment,
        });
        let response = self.client.batch_put_start(request).await?.into_inner();
        Ok((response.error_code == 0, response.batch_replica_list))
    }

    pub async fn batch_put_end(&mut self, keys: Vec<String>) -> Result<bool, ClientError> {
        let request = Request::new(BatchPutEndRequest { keys });
        let response = self.client.batch_put_end(request).await?.into_inner();
        Ok(response.error_code == 0)
    }

    pub async fn remove(&mut self, key: String) -> Result<bool, ClientError> {
        let request = Request::new(RemoveRequest { key });
        let response = self.client.remove(request).await?.into_inner();
        Ok(response.error_code == 0)
    }

    pub async fn remove_all(&mut self) -> Result<i64, ClientError> {
        let request = Request::new(RemoveAllRequest {});
        let response = self.client.remove_all(request).await?.into_inner();
        Ok(response.removed_count)
    }

    pub async fn mount_segment(&mut self, segment: Segment, client_id: String) -> Result<bool, ClientError> {
        let request = Request::new(MountSegmentRequest {
            segment_id: segment.id,
            hostname: segment.hostname,
            base: segment.base as i64,
            size: segment.size as i64,
            client_id,
        });
        let response = self.client.mount_segment(request).await?.into_inner();
        Ok(response.error_code == 0)
    }

    pub async fn unmount_segment(&mut self, segment_id: String, client_id: String) -> Result<bool, ClientError> {
        let request = Request::new(UnmountSegmentRequest { segment_id, client_id });
        let response = self.client.unmount_segment(request).await?.into_inner();
        Ok(response.error_code == 0)
    }

    pub async fn exist_key(&mut self, key: String) -> Result<bool, ClientError> {
        let request = Request::new(ExistKeyRequest { key });
        let response = self.client.exist_key(request).await?.into_inner();
        Ok(response.error_code == 0)
    }

    pub async fn batch_exist_key(&mut self, keys: Vec<String>) -> Result<Vec<bool>, ClientError> {
        let request = Request::new(BatchExistKeyRequest { keys });
        let response = self.client.batch_exist_key(request).await?.into_inner();
        Ok(response.exist_responses.into_iter().map(|code| code == 0).collect())
    }
}

impl Client {
    pub fn new(local_hostname: String, metadata_connstring: String, vault: Arc<ZetaVaultSynergy>) -> Arc<Self> {
        let client_id = Uuid::new_v4().to_string();
        log::info!("client_id={}", client_id);

        let transfer_engine = Arc::new(TransferEngine::new(
            metadata_connstring.clone(),
            local_hostname.clone(),
            "127.0.0.1".to_string(),
            8081,
            Arc::clone(&vault),
        ).unwrap());

        let client = Arc::new(Client {
            local_hostname,
            metadata_connstring,
            client_id,
            transfer_engine,
            mounted_segments: Mutex::new(Vec::new()),
            vault,
            ping_running: std::sync::atomic::AtomicBool::new(true),
            master_client: MasterClient::connect("http://master:50051".to_string()).await.unwrap(),
        });

        tokio::spawn(ping_task::ping_task(Arc::clone(&client)));

        client
    }

    pub async fn create(local_hostname: String, metadata_connstring: String, protocol: String, protocol_args: Option<*mut u8>, master_server_entry: String) -> Result<Arc<Self>, ClientError> {
        let client = Self::new(local_hostname.clone(), metadata_connstring.clone(), Arc::new(ZetaVaultSynergy::new()));
        client.connect_to_master(master_server_entry).await?;
        client.init_transfer_engine(local_hostname, metadata_connstring, protocol, protocol_args).await?;
        Ok(client)
    }

    async fn connect_to_master(&self, master_server_entry: String) -> Result<(), ClientError> {
        if master_server_entry.starts_with("etcd://") {
            let etcd_entry = &master_server_entry[7..];
            let mut etcd_client = EtcdClient::connect(["http://127.0.0.1:2379"], None).await?;
            let (master_address, _) = self.get_new_master_address(&mut etcd_client).await?;
            self.master_client.reconnect(master_address).await?;
        } else {
            self.master_client.reconnect(master_server_entry).await?;
        }
        Ok(())
    }

    async fn init_transfer_engine(&self, local_hostname: String, metadata_connstring: String, protocol: String, protocol_args: Option<*mut u8>) -> Result<(), ClientError> {
        self.transfer_engine.install_transport(protocol.clone(), "".to_string()).await?;
        self.transfer_engine.set_auto_discover(std::env::var("MC_MS_AUTO_DISC").map_or(false, |v| v == "1"));
        if self.transfer_engine.get_auto_discover() {
            if let Ok(filters) = std::env::var("MC_MS_FILTERS") {
                self.transfer_engine.set_whitelist_filters(filters.split(',').map(|s| s.trim().to_string()).collect());
            }
        }
        log::info!("Initialized transfer engine with protocol {}", protocol);
        Ok(())
    }

    async fn get_new_master_address(&self, etcd_client: &mut EtcdClient) -> Result<(String, u64), ClientError> {
        let response = etcd_client.get("master/address", None).await?.kvs().first().ok_or(ClientError::Etcd(etcd_client::Error::NotFound))?;
        let master_address = String::from_utf8(response.value().to_vec())?;
        Ok((master_address, 0)) // Placeholder for version
    }

    pub async fn get(&self, object_key: String, slices: &mut Vec<Slice>) -> Result<(), ClientError> {
        let object_info = self.query(object_key.clone()).await?;
        self.get_with_info(object_key, object_info, slices).await
    }

    async fn query(&self, object_key: String) -> Result<Vec<ReplicaDescriptor>, ClientError> {
        self.master_client.get_replica_list(object_key).await
    }

    async fn get_with_info(&self, object_key: String, object_info: Vec<ReplicaDescriptor>, slices: &mut Vec<Slice>) -> Result<(), ClientError> {
        let handles = self.find_first_complete_replica(&object_info)?;
        let total_size: usize = handles.iter().map(|h| h.size as usize).sum();
        let mut allocated_size = 0;
        for slice in slices.iter_mut() {
            allocated_size += slice.size;
        }
        if allocated_size < total_size {
            return Err(ClientError::Transfer(format!("Slice size {} is smaller than total size {}", allocated_size, total_size)));
        }
        self.transfer_engine.submit_transfer(0, vec![TransferRequest {
            opcode: TransferRequest::READ,
            source: 0,
            target_id: 0,
            target_offset: 0,
            length: total_size as u64,
        }]).await?;
        Ok(())
    }

    pub async fn put(&self, key: String, slices: &mut Vec<Slice>, config: ReplicateConfig) -> Result<(), ClientError> {
        let slice_lengths: Vec<usize> = slices.iter().map(|s| s.size).collect();
        let slice_size: usize = slice_lengths.iter().sum();
        let (success, replica_list) = self.master_client.put_start(key.clone(), slice_lengths, slice_size, config).await?;
        if success {
            let handles: Vec<AllocatedBufferDescriptor> = replica_list.into_iter().flat_map(|r| r.buffer_descriptors).collect();
            let future = self.transfer_engine.submit(handles, slices.clone(), TransferRequest::WRITE).await?;
            if future.get().await != TransferStatus::Completed {
                self.master_client.put_end(key.clone()).await?;
                return Err(ClientError::Transfer("Transfer failed".to_string()));
            }
            self.master_client.put_end(key).await?;
        }
        Ok(())
    }

    pub async fn mount_segment(&self, buffer: *mut u8, size: usize) -> Result<(), ClientError> {
        if buffer.is_null() || size == 0 {
            return Err(ClientError::Init("Invalid buffer or size".to_string()));
        }
        let segment_id = Uuid::new_v4().to_string();
        let segment = Segment {
            id: segment_id,
            hostname: self.local_hostname.clone(),
            base: buffer as usize,
            size,
        };
        self.master_client.mount_segment(segment.clone(), self.client_id.clone()).await?;
        let mut segments = self.mounted_segments.lock().unwrap();
        segments.push(segment);
        self.transfer_engine.register_local_memory(buffer as usize, size as u64, "local".to_string()).await?;
        Ok(())
    }

    pub async fn unmount_segment(&self, buffer: *mut u8, size: usize) -> Result<(), ClientError> {
        let mut segments = self.mounted_segments.lock().unwrap();
        if let Some(index) = segments.iter().position(|s| s.base == buffer as usize && s.size == size) {
            let segment = segments.remove(index);
            self.master_client.unmount_segment(segment.id, self.client_id.clone()).await?;
            self.transfer_engine.unregister_local_memory(buffer as usize).await?;
        }
        Ok(())
    }

    fn find_first_complete_replica(&self, replica_list: &[ReplicaDescriptor]) -> Result<Vec<AllocatedBufferDescriptor>, ClientError> {
        for replica in replica_list {
            if replica.status == 1 { // COMPLETE
                return Ok(replica.buffer_descriptors.clone());
            }
        }
        Err(ClientError::Transfer("No complete replica found".to_string()))
    }

    async fn remount_segments(&self, etcd_client: &mut EtcdClient) -> Result<(), ClientError> {
        let segments = self.mounted_segments.lock().unwrap().clone();
        let request = Request::new(ReMountSegmentRequest {
            segments: segments.into_iter().map(|s| mooncake::Segment {
                id: s.id,
                hostname: s.hostname,
                base: s.base as i64,
                size: s.size as i64,
            }).collect(),
            client_id: self.client_id.clone(),
        });
        let response = self.master_client.client.re_mount_segment(request).await?.into_inner();
        if response.error_code != 0 {
            Err(ClientError::Transfer(format!("Remount failed: {}", response.error_code)))
        } else {
            Ok(())
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.ping_running.store(false, std::sync::atomic::Ordering::SeqCst);
        let segments = self.mounted_segments.lock().unwrap().clone();
        for segment in segments {
            self.unmount_segment(segment.base as *mut u8, segment.size).ok();
        }
    }
}