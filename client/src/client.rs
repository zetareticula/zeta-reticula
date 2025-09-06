 // Copyright 2025 zeta-reticula
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
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use log;
use uuid::Uuid;
use p2pstore::{TransferEngineError, AllocatedBufferDescriptor};
use zeta_vault_synergy::{ZetaVaultSynergy, VaultConfig};
use etcd_client::Client as EtcdClient;
use tonic::{transport::Channel, Request};
use crate::ping_task;
use self::mooncake::{
    ReplicaDescriptor,
    BatchExistKeyRequest,
    PingResponse,
    PingRequest,
    GetReplicaListRequest,
    BatchGetReplicaListRequest,
    PutStartRequest,
    PutEndRequest,
    BatchPutStartRequest,
    BatchPutEndRequest,
    RemoveRequest,
    RemoveAllRequest,
    MountSegmentRequest,
    UnmountSegmentRequest,
    ReMountSegmentRequest,
};

// Local stub for Mooncake gRPC types to allow compilation without generated code.
// Replace with `tonic::include_proto!("mooncake");` and proper build.rs when available.
mod mooncake {
    use ::p2pstore::AllocatedBufferDescriptor;
    use tonic::{Response, Status};
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct PingRequest {
        pub client_id: String,
    }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct PingResponse {
        pub client_status: i32,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct GetReplicaListRequest {
        pub key: String,
    }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct ReplicaDescriptor {
        pub status: i32,
        pub buffer_descriptors: Vec<AllocatedBufferDescriptor>,
    }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct BatchGetReplicaListRequest {
        pub keys: Vec<String>,
    }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct BatchReplicaListResponse {
        pub batch_replica_list: std::collections::HashMap<String, Vec<ReplicaDescriptor>>,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct PutStartRequest {
        pub key: String,
        pub slice_lengths: Vec<usize>,
        pub total_size: i64,
        pub replica_num: i32,
        pub preferred_segment: String,
    }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct PutStartResponse {
        pub error_code: i32,
        pub replica_list: Vec<ReplicaDescriptor>,
    }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct PutEndRequest {
        pub key: String,
    }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct GenericBoolResponse {
        pub error_code: i32,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct BatchPutStartRequest {
        pub keys: Vec<String>,
        pub value_lengths: std::collections::HashMap<String, i64>,
        pub slice_lengths: std::collections::HashMap<String, Vec<usize>>,
        pub replica_num: i32,
        pub preferred_segment: String,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct BatchPutEndRequest {
        pub keys: Vec<String>,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct RemoveRequest {
        pub key: String,
    }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct RemoveAllRequest {}

    // Exist key API
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct BatchExistKeyRequest { pub keys: Vec<String> }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct BatchExistKeyResponse { pub exist_responses: Vec<i32> }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct Segment {
        pub id: String,
        pub hostname: String,
        pub base: i64,
        pub size: i64,
    }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct MountSegmentRequest {
        pub segment_id: String,
        pub hostname: String,
        pub base: i64,
        pub size: i64,
        pub client_id: String,
    }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct UnmountSegmentRequest {
        pub segment_id: String,
        pub client_id: String,
    }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct ReMountSegmentRequest {
        pub segments: Vec<Segment>,
        pub client_id: String,
    }

    pub mod master_service_client {
        use super::*;
        use tonic::{Request, Response, Status};

        #[derive(Debug, Clone, Default)]
        pub struct MasterServiceClient<T>(std::marker::PhantomData<T>);

        impl<T> MasterServiceClient<T> {
            pub fn new(_channel: T) -> Self { Self(Default::default()) }

            pub async fn ping(&mut self, _req: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
                Ok(Response::new(PingResponse { client_status: 0 }))
            }
            pub async fn get_replica_list(&mut self, _req: Request<GetReplicaListRequest>) -> Result<Response<super::ReplicaListResponse>, Status> {
                Ok(Response::new(super::ReplicaListResponse { replica_list: vec![] }))
            }
            pub async fn batch_get_replica_list(&mut self, _req: Request<BatchGetReplicaListRequest>) -> Result<Response<super::BatchReplicaListResponse>, Status> {
                Ok(Response::new(BatchReplicaListResponse { batch_replica_list: Default::default() }))
            }
            pub async fn put_start(&mut self, _req: Request<PutStartRequest>) -> Result<Response<PutStartResponse>, Status> {
                Ok(Response::new(PutStartResponse { error_code: 0, replica_list: vec![] }))
            }
            pub async fn put_end(&mut self, _req: Request<PutEndRequest>) -> Result<Response<GenericBoolResponse>, Status> {
                Ok(Response::new(GenericBoolResponse { error_code: 0 }))
            }
            pub async fn batch_put_start(&mut self, _req: Request<BatchPutStartRequest>) -> Result<Response<super::BatchPutStartResponse>, Status> {
                Ok(Response::new(super::BatchPutStartResponse { error_code: 0, batch_replica_list: Default::default() }))
            }
            pub async fn batch_put_end(&mut self, _req: Request<BatchPutEndRequest>) -> Result<Response<GenericBoolResponse>, Status> {
                Ok(Response::new(GenericBoolResponse { error_code: 0 }))
            }
            pub async fn remove(&mut self, _req: Request<RemoveRequest>) -> Result<Response<GenericBoolResponse>, Status> {
                Ok(Response::new(GenericBoolResponse { error_code: 0 }))
            }
            pub async fn remove_all(&mut self, _req: Request<RemoveAllRequest>) -> Result<Response<super::RemoveAllResponse>, Status> {
                Ok(Response::new(super::RemoveAllResponse { removed_count: 0 }))
            }
            pub async fn batch_exist_key(&mut self, _req: Request<BatchExistKeyRequest>) -> Result<Response<super::BatchExistKeyResponse>, Status> {
                Ok(Response::new(super::BatchExistKeyResponse { exist_responses: vec![] }))
            }
            pub async fn mount_segment(&mut self, _req: Request<MountSegmentRequest>) -> Result<Response<GenericBoolResponse>, Status> {
                Ok(Response::new(GenericBoolResponse { error_code: 0 }))
            }
            pub async fn unmount_segment(&mut self, _req: Request<UnmountSegmentRequest>) -> Result<Response<GenericBoolResponse>, Status> {
                Ok(Response::new(GenericBoolResponse { error_code: 0 }))
            }
            pub async fn re_mount_segment(&mut self, _req: Request<ReMountSegmentRequest>) -> Result<Response<GenericBoolResponse>, Status> {
                Ok(Response::new(GenericBoolResponse { error_code: 0 }))
            }
        }
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct ReplicaListResponse { pub replica_list: Vec<ReplicaDescriptor> }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct BatchPutStartResponse { pub error_code: i32, pub batch_replica_list: std::collections::HashMap<String, Vec<ReplicaDescriptor>> }
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct RemoveAllResponse { pub removed_count: i64 }
}

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

#[derive(Debug)]
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

#[derive(Debug, Clone)]
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
    mounted_segments: Mutex<Vec<Segment>>,
    vault: Arc<ZetaVaultSynergy>,
    ping_running: std::sync::atomic::AtomicBool,
    master_client: RwLock<MasterClient>,
}

pub struct MasterClient {
    channel: Channel,
    client: mooncake::master_service_client::MasterServiceClient<Channel>,
}

impl MasterClient {
    pub async fn connect(address: String) -> Result<Self, ClientError> {
        let channel = Channel::from_shared(address)
            .map_err(|e| ClientError::Init(e.to_string()))?
            .connect()
            .await
            .map_err(|e| ClientError::Init(e.to_string()))?;
        let client = mooncake::master_service_client::MasterServiceClient::new(channel.clone());
        Ok(MasterClient { channel, client })
    }

    pub async fn reconnect(&mut self, address: String) -> Result<(), ClientError> {
        self.channel = Channel::from_shared(address)
            .map_err(|e| ClientError::Init(e.to_string()))?
            .connect()
            .await
            .map_err(|e| ClientError::Init(e.to_string()))?;
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
        // Fallback implementation using batch_exist_key
        let request = Request::new(BatchExistKeyRequest { keys: vec![key] });
        let response = self.client.batch_exist_key(request).await?.into_inner();
        Ok(response.exist_responses.first().copied().unwrap_or(1) == 0)
    }

    pub async fn batch_exist_key(&mut self, keys: Vec<String>) -> Result<Vec<bool>, ClientError> {
        let request = Request::new(BatchExistKeyRequest { keys });
        let response = self.client.batch_exist_key(request).await?.into_inner();
        Ok(response.exist_responses.into_iter().map(|code| code == 0).collect())
    }
}

impl Client {
    pub async fn new(local_hostname: String, metadata_connstring: String) -> Arc<Self> {
        let client_id = Uuid::new_v4().to_string();
        log::info!("client_id={}", client_id);
        let vault = Arc::new(ZetaVaultSynergy::new(1, VaultConfig::default()).await.expect("init ZetaVaultSynergy"));
        let client = Arc::new(Client {
            local_hostname,
            metadata_connstring,
            client_id,
            mounted_segments: Mutex::new(Vec::new()),
            vault,
            ping_running: std::sync::atomic::AtomicBool::new(true),
            master_client: RwLock::new(MasterClient::connect("http://master:50051".to_string()).await.unwrap()),
        });

        tokio::spawn(ping_task::ping_task(Arc::clone(&client)));

        client
    }

    pub async fn create(local_hostname: String, metadata_connstring: String, protocol: String, protocol_args: Option<*mut u8>, master_server_entry: String) -> Result<Arc<Self>, ClientError> {
        let client = Self::new(local_hostname.clone(), metadata_connstring.clone()).await;
        client.connect_to_master(master_server_entry).await?;
        client.init_transfer_engine(local_hostname, metadata_connstring, protocol, protocol_args).await?;
        Ok(client)
    }

    async fn connect_to_master(&self, master_server_entry: String) -> Result<(), ClientError> {
        if master_server_entry.starts_with("etcd://") {
            let etcd_entry = &master_server_entry[7..];
            let mut etcd_client = EtcdClient::connect(["http://127.0.0.1:2379"], None).await?;
            let (master_address, _) = self.get_new_master_address(&mut etcd_client).await?;
            self.master_client.write().await.reconnect(master_address).await?;
        } else {
            self.master_client.write().await.reconnect(master_server_entry).await?;
        }
        Ok(())
    }

    async fn init_transfer_engine(&self, _local_hostname: String, _metadata_connstring: String, protocol: String, _protocol_args: Option<*mut u8>) -> Result<(), ClientError> {
        // No-op stub: transfer engine is not wired here; initialization is skipped.
        log::info!("Initialized transfer engine with protocol {} (stub)", protocol);
        Ok(())
    }

    pub async fn get_new_master_address(&self, etcd_client: &mut EtcdClient) -> Result<(String, u64), ClientError> {
        let get_resp = etcd_client.get("master/address", None).await?;
        let kv = get_resp.kvs().first().ok_or_else(|| ClientError::Init("master/address not found".to_string()))?;
        let master_address = String::from_utf8(kv.value().to_vec()).map_err(|e| ClientError::Init(format!("utf8 error: {}", e)))?;
        Ok((master_address, 0)) // Placeholder for version
    }

    pub async fn get(&self, object_key: String, slices: &mut Vec<Slice>) -> Result<(), ClientError> {
        let object_info = self.query(object_key.clone()).await?;
        self.get_with_info(object_key, object_info, slices).await
    }

    async fn query(&self, object_key: String) -> Result<Vec<ReplicaDescriptor>, ClientError> {
        self.master_client.write().await.get_replica_list(object_key).await
    }

    async fn get_with_info(&self, _object_key: String, object_info: Vec<ReplicaDescriptor>, slices: &mut Vec<Slice>) -> Result<(), ClientError> {
        let _handles = self.find_first_complete_replica(&object_info)?;
        let total_size: usize = object_info.iter().flat_map(|r| r.buffer_descriptors.iter()).map(|h| h.size_ as usize).sum();
        let mut allocated_size = 0;
        for slice in slices.iter_mut() {
            allocated_size += slice.size;
        }
        if allocated_size < total_size {
            return Err(ClientError::Transfer(format!("Slice size {} is smaller than total size {}", allocated_size, total_size)));
        }
        // No-op: a real implementation would issue transfer requests here.
        Ok(())
    }

    pub async fn put(&self, key: String, slices: &mut Vec<Slice>, config: ReplicateConfig) -> Result<(), ClientError> {
        let slice_lengths: Vec<usize> = slices.iter().map(|s| s.size).collect();
        let slice_size: usize = slice_lengths.iter().sum();
        let (success, _replica_list) = self.master_client.write().await.put_start(key.clone(), slice_lengths, slice_size, config).await?;
        if success {
            // No-op: transfer submission is stubbed out.
            self.master_client.write().await.put_end(key).await?;
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
        self.master_client.write().await.mount_segment(segment.clone(), self.client_id.clone()).await?;
        let mut segments = self.mounted_segments.lock().unwrap();
        segments.push(segment);
        // No-op: local memory registration stubbed.
        Ok(())
    }

    pub async fn unmount_segment(&self, buffer: *mut u8, size: usize) -> Result<(), ClientError> {
        let mut segments = self.mounted_segments.lock().unwrap();
        if let Some(index) = segments.iter().position(|s| s.base == buffer as usize && s.size == size) {
            let segment = segments.remove(index);
            self.master_client.write().await.unmount_segment(segment.id, self.client_id.clone()).await?;
            // No-op: local memory unregistration stubbed.
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

    pub async fn remount_segments(&self, _etcd_client: &mut EtcdClient) -> Result<(), ClientError> {
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
        let response = self.master_client.write().await.client.re_mount_segment(request).await?.into_inner();
        if response.error_code != 0 {
            Err(ClientError::Transfer(format!("Remount failed: {}", response.error_code)))
        } else {
            Ok(())
        }
    }

    // Public helpers for ping_task
    pub async fn ping_master(&self, client_id: String) -> Result<PingResponse, ClientError> {
        self.master_client.write().await.ping(client_id).await
    }
    pub async fn reconnect_master(&self, addr: String) -> Result<(), ClientError> {
        self.master_client.write().await.reconnect(addr).await
    }
    pub fn client_id(&self) -> String { self.client_id.clone() }
    pub fn is_running(&self) -> bool { self.ping_running.load(std::sync::atomic::Ordering::SeqCst) }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.ping_running.store(false, std::sync::atomic::Ordering::SeqCst);
        // Note: cannot call async code in Drop; cleanup should be handled elsewhere.
    }
}