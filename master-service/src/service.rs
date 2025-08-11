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
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use parking_lot::Mutex as ParkingMutex;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use log;
use rand::Rng;
use uuid::Uuid;
use p2pstore::{ReplicaStatus, AllocatedBufferDescriptor};
use zeta_vault_synergy::ZetaVaultSynergy;
use crate::gc_task::GCTask;

const K_NUM_SHARDS: usize = 16;
const K_MAX_SLICE_SIZE: u64 = 1024 * 1024; // 1MB
const K_GC_THREAD_SLEEP_MS: u64 = 1000;
const K_CLIENT_MONITOR_SLEEP_MS: u64 = 1000;

// Error types for the master service
#[derive(Error, Debug)]
pub enum MasterServiceError {
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Object not found: {0}")]
    ObjectNotFound(String),
    #[error("Replica not ready: {0}")]
    ReplicaNotReady(String),
    #[error("Segment not found: {0}")]
    SegmentNotFound(String),
}

// Segment structure for the master service
#[derive(Debug, Serialize, Deserialize)]
pub struct Segment {
    id: String,
    name: String,
    client_id: String,
}

// Replica structure for the master service
#[derive(Debug, Serialize, Deserialize)]
pub struct Replica {
    // Handles for the replica
    handles: Vec<AllocatedBufferDescriptor>,
    status: i32, // 0: PROCESSING, 1: COMPLETE
}

// Implementation for the Replica structure
impl Replica {
    pub fn new(handles: Vec<AllocatedBufferDescriptor>) -> Self {
        Replica {
            handles,
            status: 0, // PROCESSING
        }
    }

    pub fn mark_complete(&mut self) {
        self.status = 1; // COMPLETE
    }

    pub fn get_descriptor(&self) -> Vec<AllocatedBufferDescriptor> {
        self.handles.clone()
    }

    pub fn has_invalid_handle(&self) -> bool {
        self.handles.iter().any(|h| h.buffer_address_ == 0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectMetadata {
    size: u64,
    replicas: Vec<Replica>,
    lease_timeout: Instant,
}

impl ObjectMetadata {
    pub fn new(size: u64, replicas: Vec<Replica>) -> Self {
        ObjectMetadata {
            size,
            replicas,
            lease_timeout: Instant::now(),
        }
    }

    pub fn grant_lease(&mut self, ttl_ms: u64) {
        self.lease_timeout = Instant::now() + Duration::from_millis(ttl_ms);
    }

    pub fn is_lease_expired(&self) -> bool {
        Instant::now() >= self.lease_timeout
    }

    pub fn has_diff_rep_status(&self, status: i32) -> Option<i32> {
        if self.replicas.iter().any(|r| r.status != status) {
            Some(status)
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplicateConfig {
    replica_num: usize,
    preferred_segment: String,
}

pub struct MasterService {
    allocation_strategy: Arc<dyn AllocationStrategy>,
    enable_gc: bool,
    default_kv_lease_ttl: u64,
    eviction_ratio: f64,
    eviction_high_watermark_ratio: f64,
    client_live_ttl_sec: i64,
    enable_ha: bool,
    gc_running: bool,
    client_monitor_running: bool,
    gc_queue: Arc<Mutex<Vec<GCTask>>>,
    client_ping_queue: Arc<Mutex<Vec<Uuid>>>,
    metadata_shards: Vec<MetadataShard>,
    segment_manager: Arc<SegmentManager>,
    need_eviction: bool,
    ok_clients: RwLock<Vec<Uuid>>,
    view_version: u64,
}

struct MetadataShard {
    metadata: ParkingMutex<std::collections::HashMap<String, ObjectMetadata>>,
}

struct SegmentManager {
    // Placeholder for segment management logic
}

impl SegmentManager {
    fn get_segment_access(&self) -> ScopedSegmentAccess {
        ScopedSegmentAccess {}
    }
}

struct ScopedSegmentAccess;

impl ScopedSegmentAccess {
    fn mount_segment(&self, segment: Segment, client_id: String) -> Result<(), MasterServiceError> {
        // Placeholder implementation
        Ok(())
    }

    fn remount_segment(&self, segments: Vec<Segment>, client_id: String) -> Result<(), MasterServiceError> {
        // Placeholder implementation
        Ok(())
    }

    fn prepare_unmount_segment(&self, segment_id: String, metrics_dec_capacity: &mut usize) -> Result<(), MasterServiceError> {
        // Placeholder implementation
        Ok(())
    }

    fn commit_unmount_segment(&self, segment_id: String, client_id: String, metrics_dec_capacity: usize) -> Result<(), MasterServiceError> {
        // Placeholder implementation
        Ok(())
    }

    fn get_all_segments(&self, all_segments: &mut Vec<String>) -> Result<(), MasterServiceError> {
        // Placeholder implementation
        Ok(())
    }

    fn query_segments(&self, segment: String, used: &mut usize, capacity: &mut usize) -> Result<(), MasterServiceError> {
        // Placeholder implementation
        Ok(())
    }
}

trait AllocationStrategy {
    fn allocate(&self, size: u64, config: ReplicateConfig) -> Option<AllocatedBufferDescriptor>;
}

struct RandomAllocationStrategy;

impl AllocationStrategy for RandomAllocationStrategy {
    fn allocate(&self, size: u64, _config: ReplicateConfig) -> Option<AllocatedBufferDescriptor> {
        if size > K_MAX_SLICE_SIZE {
            None
        } else {
            Some(AllocatedBufferDescriptor {
                buffer_address_: rand::thread_rng().gen(),
                size_: size,
            })
        }
    }
}

impl MasterService {
    pub fn new(
        enable_gc: bool,
        default_kv_lease_ttl: u64,
        eviction_ratio: f64,
        eviction_high_watermark_ratio: f64,
        view_version: u64,
        client_live_ttl_sec: i64,
        enable_ha: bool,
    ) -> Arc<Self> {
        if eviction_ratio < 0.0 || eviction_ratio > 1.0 {
            log::error!("Eviction ratio must be between 0.0 and 1.0, current value: {}", eviction_ratio);
            panic!("Invalid eviction ratio");
        }
        if eviction_high_watermark_ratio < 0.0 || eviction_high_watermark_ratio > 1.0 {
            log::error!("Eviction high watermark ratio must be between 0.0 and 1.0, current value: {}", eviction_high_watermark_ratio);
            panic!("Invalid eviction high watermark ratio");
        }

        let service = Arc::new(MasterService {
            allocation_strategy: Arc::new(RandomAllocationStrategy),
            enable_gc,
            default_kv_lease_ttl,
            eviction_ratio,
            eviction_high_watermark_ratio,
            client_live_ttl_sec,
            enable_ha,
            gc_running: true,
            client_monitor_running: enable_ha,
            gc_queue: Arc::new(Mutex::new(Vec::new())),
            client_ping_queue: Arc::new(Mutex::new(Vec::new())),
            metadata_shards: (0..K_NUM_SHARDS).map(|_| MetadataShard {
                metadata: ParkingMutex::new(std::collections::HashMap::new()),
            }).collect(),
            segment_manager: Arc::new(SegmentManager {}),
            need_eviction: false,
            ok_clients: RwLock::new(Vec::new()),
            view_version,
        });

        if enable_gc {
            let service_clone = Arc::clone(&service);
            tokio::spawn(async move {
                service_clone.gc_thread_func().await;
            });
        }

        if enable_ha {
            let service_clone = Arc::clone(&service);
            tokio::spawn(async move {
                service_clone.client_monitor_func().await;
            });
        }

        service
    }

    pub async fn mount_segment(&self, segment: Segment, client_id: String) -> Result<(), MasterServiceError> {
        let segment_access = self.segment_manager.get_segment_access();
        if self.enable_ha {
            let mut ping_queue = self.client_ping_queue.lock().unwrap();
            if ping_queue.len() >= 1000 { // Arbitrary limit
                log::error!("segment_name={}, error=client_ping_queue_full", segment.name);
                return Err(MasterServiceError::Internal("Client ping queue full".to_string()));
            }
            ping_queue.push(Uuid::parse_str(&client_id).unwrap());
        }
        let result = segment_access.mount_segment(segment, client_id);
        if result.is_ok() || result == Err(MasterServiceError::SegmentNotFound("")) {
            Ok(())
        } else {
            result
        }
    }

    pub async fn remount_segment(&self, segments: Vec<Segment>, client_id: String) -> Result<(), MasterServiceError> {
        if !self.enable_ha {
            log::error!("ReMountSegment is only available in HA mode");
            return Err(MasterServiceError::Internal("Unavailable in current mode".to_string()));
        }

        let mut ok_clients = self.ok_clients.write().await;
        if ok_clients.contains(&Uuid::parse_str(&client_id).unwrap()) {
            log::warn!("client_id={}, warn=client_already_remounted", client_id);
            return Ok(());
        }

        let segment_access = self.segment_manager.get_segment_access();
        if self.enable_ha {
            let mut ping_queue = self.client_ping_queue.lock().unwrap();
            if ping_queue.len() >= 1000 {
                log::error!("client_id={}, error=client_ping_queue_full", client_id);
                return Err(MasterServiceError::Internal("Client ping queue full".to_string()));
            }
            ping_queue.push(Uuid::parse_str(&client_id).unwrap());
        }
        let result = segment_access.remount_segment(segments, client_id);
        if result.is_ok() {
            ok_clients.push(Uuid::parse_str(&client_id).unwrap());
        }
        result
    }

    pub fn clear_invalid_handles(&self) {
        for shard in &self.metadata_shards {
            let mut metadata = shard.metadata.lock();
            let mut to_remove = Vec::new();
            for (key, metadata) in metadata.iter() {
                let has_invalid = metadata.replicas.iter().any(|r| r.has_invalid_handle());
                if has_invalid || self.cleanup_stale_handles(metadata.clone()) {
                    to_remove.push(key.clone());
                }
            }
            for key in to_remove {
                metadata.remove(&key);
            }
        }
    }

    pub async fn unmount_segment(&self, segment_id: String, client_id: String) -> Result<(), MasterServiceError> {
        let mut metrics_dec_capacity = 0;
        {
            let segment_access = self.segment_manager.get_segment_access();
            let result = segment_access.prepare_unmount_segment(segment_id.clone(), &mut metrics_dec_capacity);
            if result == Err(MasterServiceError::SegmentNotFound("")) {
                return Ok(());
            }
            if result.is_err() {
                return result;
            }
        }

        self.clear_invalid_handles();

        let segment_access = self.segment_manager.get_segment_access();
        segment_access.commit_unmount_segment(segment_id, client_id, metrics_dec_capacity)
    }

    pub async fn exist_key(&self, key: String) -> Result<(), MasterServiceError> {
        let shard_idx = self.get_shard_index(&key);
        let mut metadata = self.metadata_shards[shard_idx].metadata.lock();
        if let Some(metadata) = metadata.get_mut(&key) {
            if let Some(status) = metadata.has_diff_rep_status(1) { // COMPLETE
                log::warn!("key={}, status={}, error=replica_not_ready", key, status);
                return Err(MasterServiceError::ReplicaNotReady(format!("Replica not ready: {}", status)));
            }
            metadata.grant_lease(self.default_kv_lease_ttl);
            Ok(())
        } else {
            log::info!("key={}, info=object_not_found", key);
            Err(MasterServiceError::ObjectNotFound(key))
        }
    }

    pub async fn get_replica_list(&self, key: String, replica_list: &mut Vec<ReplicaDescriptor>) -> Result<(), MasterServiceError> {
        let shard_idx = self.get_shard_index(&key);
        let mut metadata = self.metadata_shards[shard_idx].metadata.lock();
        if let Some(metadata) = metadata.get_mut(&key) {
            if let Some(status) = metadata.has_diff_rep_status(1) { // COMPLETE
                log::warn!("key={}, status={}, error=replica_not_ready", key, status);
                return Err(MasterServiceError::ReplicaNotReady(format!("Replica not ready: {}", status)));
            }
            replica_list.clear();
            for replica in &metadata.replicas {
                replica_list.push(replica.get_descriptor());
            }
            if self.enable_gc {
                self.mark_for_gc(key.clone(), 1000).await?;
            } else {
                metadata.grant_lease(self.default_kv_lease_ttl);
            }
            Ok(())
        } else {
            log::info!("key={}, info=object_not_found", key);
            Err(MasterServiceError::ObjectNotFound(key))
        }
    }

    pub async fn put_start(&self, key: String, value_length: u64, slice_lengths: Vec<u64>, config: ReplicateConfig, replica_list: &mut Vec<ReplicaDescriptor>) -> Result<(), MasterServiceError> {
        if config.replica_num == 0 || value_length == 0 || key.is_empty() {
            log::error!("key={}, replica_num={}, value_length={}, key_size={}, error=invalid_params", key, config.replica_num, value_length, key.len());
            return Err(MasterServiceError::InvalidParam("Invalid parameters".to_string()));
        }

        let total_length: u64 = slice_lengths.iter().sum();
        if total_length != value_length {
            log::error!("key={}, total_length={}, expected_length={}, error=slice_length_mismatch", key, total_length, value_length);
            return Err(MasterServiceError::InvalidParam("Slice length mismatch".to_string()));
        }

        let shard_idx = self.get_shard_index(&key);
        let mut metadata = self.metadata_shards[shard_idx].metadata.lock();
        if let Some(existing) = metadata.get_mut(&key) {
            if !self.cleanup_stale_handles(existing.clone()) {
                log::info!("key={}, info=object_already_exists", key);
                return Ok(());
            }
        }

        let mut replicas = Vec::with_capacity(config.replica_num);
        for _ in 0..config.replica_num {
            let mut handles = Vec::with_capacity(slice_lengths.len());
            for &chunk_size in &slice_lengths {
                if chunk_size > K_MAX_SLICE_SIZE {
                    log::error!("key={}, slice_index={}, slice_size={}, max_size={}, error=invalid_slice_size", key, handles.len(), chunk_size, K_MAX_SLICE_SIZE);
                    return Err(MasterServiceError::InvalidParam("Invalid slice size".to_string()));
                }
                if let Some(handle) = self.allocation_strategy.allocate(chunk_size, config.clone()) {
                    handles.push(handle);
                } else {
                    log::error!("key={}, replica_id={}, slice_index={}, error=allocation_failed", key, replicas.len(), handles.len());
                    self.need_eviction = true;
                    return Err(MasterServiceError::Internal("No available handle".to_string()));
                }
            }
            replicas.push(Replica::new(handles));
        }

        let metadata = ObjectMetadata::new(value_length, replicas);
        metadata_shards[shard_idx].metadata.insert(key.clone(), metadata);
        Ok(())
    }

    pub async fn put_end(&self, key: String) -> Result<(), MasterServiceError> {
        let shard_idx = self.get_shard_index(&key);
        let mut metadata = self.metadata_shards[shard_idx].metadata.lock();
        if let Some(metadata) = metadata.get_mut(&key) {
            for replica in &mut metadata.replicas {
                replica.mark_complete();
            }
            metadata.grant_lease(0);
            Ok(())
        } else {
            log::error!("key={}, error=object_not_found", key);
            Err(MasterServiceError::ObjectNotFound(key))
        }
    }

    fn get_shard_index(&self, key: &str) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % K_NUM_SHARDS
    }

    async fn mark_for_gc(&self, key: String, delay_ms: u64) -> Result<(), MasterServiceError> {
        let mut gc_queue = self.gc_queue.lock().unwrap();
        if gc_queue.len() >= 1000 { // Arbitrary limit
            log::error!("key={}, error=gc_queue_full", key);
            return Err(MasterServiceError::Internal("GC queue full".to_string()));
        }
        gc_queue.push(GCTask::new(key, delay_ms));
        Ok(())
    }

    fn cleanup_stale_handles(&self, mut metadata: ObjectMetadata) -> bool {
        metadata.replicas.retain(|r| !r.has_invalid_handle());
        metadata.replicas.is_empty()
    }

    async fn gc_thread_func(&self) {
        log::info!("action=gc_thread_started");
        let mut local_pq = std::collections::BinaryHeap::new();

        while self.gc_running {
            {
                let mut gc_queue = self.gc_queue.lock().unwrap();
                while let Some(task) = gc_queue.pop() {
                    local_pq.push(task);
                }
            }

            while let Some(task) = local_pq.peek() {
                if !task.is_ready() {
                    break;
                }
                let task = local_pq.pop().unwrap();
                log::info!("key={}, action=gc_removing_key", task.get_key());
                let result = self.remove(task.get_key().to_string()).await;
                if result.is_err() && result != Err(MasterServiceError::ObjectNotFound("")) && result != Err(MasterServiceError::ReplicaNotReady("")) {
                    log::warn!("key={}, error=gc_remove_failed, error_code={:?}", task.get_key(), result);
                }
            }

            let used_ratio = 0.5; // Placeholder for metric
            if used_ratio > self.eviction_high_watermark_ratio || (self.need_eviction && self.eviction_ratio > 0.0) {
                self.batch_evict(self.eviction_ratio).await;
            }

            tokio::time::sleep(Duration::from_millis(K_GC_THREAD_SLEEP_MS)).await;
        }

        while let Some(task) = local_pq.pop() {
            drop(task); // Ensure cleanup
        }
        log::info!("action=gc_thread_stopped");
    }

    async fn batch_evict(&self, eviction_ratio: f64) {
        let now = Instant::now();
        let mut evicted_count = 0;
        let mut object_count = 0;
        let mut total_freed_size = 0;

        let start_idx = rand::thread_rng().gen_range(0..K_NUM_SHARDS);
        for i in 0..K_NUM_SHARDS {
            let shard_idx = (start_idx + i) % K_NUM_SHARDS;
            let mut metadata = self.metadata_shards[shard_idx].metadata.lock();
            object_count += metadata.len();

            let ideal_evict_num = ((object_count as f64) * eviction_ratio - evicted_count as f64).ceil() as i64;
            if ideal_evict_num <= 0 {
                continue;
            }

            let mut candidates = Vec::new();
            for (key, meta) in metadata.iter() {
                if meta.is_lease_expired() && meta.has_diff_rep_status(1).is_none() {
                    candidates.push((key.clone(), meta.lease_timeout));
                }
            }

            if !candidates.is_empty() {
                let evict_num = std::cmp::min(ideal_evict_num as usize, candidates.len());
                candidates.sort_by_key(|(_, t)| *t);
                let target_timeout = candidates[evict_num - 1].1;
                let mut to_remove = Vec::new();
                for (key, meta) in metadata.iter_mut() {
                    if meta.lease_timeout <= target_timeout && meta.has_diff_rep_status(1).is_none() {
                        total_freed_size += meta.size * meta.replicas.len() as u64;
                        to_remove.push(key.clone());
                        evicted_count += 1;
                    }
                }
                for key in to_remove {
                    metadata.remove(&key);
                }
            }
        }

        if evicted_count > 0 {
            self.need_eviction = false;
        }
        log::info!("action=evict_objects, evicted_count={}, total_freed_size={}", evicted_count, total_freed_size);
    }
}

impl Drop for MasterService {
    fn drop(&mut self) {
        self.gc_running = false;
        self.client_monitor_running = false;
    }
}