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

use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use log;
use parking_lot::Mutex as ParkingMutex;
use p2pstore::Segment;

#[derive(Error, Debug)]
pub enum MasterServiceError {
    #[error("Segment error: {0}")]
    Segment(String),
}

pub struct MasterService {
    segments: RwLock<std::collections::HashMap<String, Segment>>,
}

impl MasterService {
    pub fn new() -> Arc<Self> {
        Arc::new(MasterService {
            segments: RwLock::new(std::collections::HashMap::new()),
        })
    }

    pub async fn mount_segment(&self, segment: Segment, client_id: String) -> Result<(), MasterServiceError> {
        let mut segments = self.segments.write().await;
        segments.insert(segment.id.clone(), segment);
        log::info!("Mounted segment {} for client {}", segment.id, client_id);
        Ok(())
    }

    pub async fn remount_segment(&self, segments: Vec<Segment>, client_id: String) -> Result<(), MasterServiceError> {
        let mut stored_segments = self.segments.write().await;
        for segment in segments {
            stored_segments.insert(segment.id.clone(), segment);
        }
        log::info!("Remounted segments for client {}", client_id);
        Ok(())
    }

    pub async fn unmount_segment(&self, segment_id: String, client_id: String) -> Result<(), MasterServiceError> {
        let mut segments = self.segments.write().await;
        if segments.remove(&segment_id).is_some() {
            log::info!("Unmounted segment {} for client {}", segment_id, client_id);
            Ok(())
        } else {
            Err(MasterServiceError::Segment(format!("Segment {} not found", segment_id)))
        }
    }
}