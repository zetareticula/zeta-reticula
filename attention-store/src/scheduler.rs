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
use tokio::sync::RwLock;
use log;

use crate::store::SessionContext;

// Scheduler manages the job queue for attention store
pub struct Scheduler {
    // Job queue with session IDs is RwLock protected because it may be modified by multiple threads
    job_queue: RwLock<Vec<String>>, // Simulated job queue with session IDs
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            job_queue: RwLock::new(Vec::new()),
        }
    }

    // Evict sessions from host memory to disk storage
    pub async fn evict(&self, look_ahead_window: usize, host_memory: &Mutex<Vec<SessionContext>>, disk_storage: &Mutex<Vec<SessionContext>>) {
        // Read job queue
        let job_queue = self.job_queue.read().await;
        let mut host_mem = host_memory.lock().unwrap();
        // Lock disk storage
        let mut disk = disk_storage.lock().unwrap();

        // if the total number of sessions exceeds the look ahead window, evict some sessions
        if host_mem.len() + disk.len() > look_ahead_window {
            //
            let mut to_evict = Vec::new();
            for ctx in &*host_mem {
                let sid = ctx.session_id().to_string();
                if !job_queue.contains(&sid) && !to_evict.contains(&sid) {
                    to_evict.push(sid);
                }
            }

            if to_evict.is_empty() {
                if let Some(last) = host_mem.iter().enumerate()
                    .find(|(_, c)| !job_queue.contains(&c.session_id().to_string()))
                    .map(|(i, _)| i) {
                    let sid = host_mem[last].session_id().to_string();
                    to_evict.push(sid);
                }
            }

            for session_id in to_evict {
                if let Some(idx) = host_mem.iter().position(|c| c.session_id() == session_id) {
                    let ctx = host_mem.remove(idx);
                    if disk.len() * 1024 < 10 * 1024 * 1024 { // 10GB disk limit
                        disk.push(ctx);
                        log::info!("Evicted session {} from host to disk", session_id);
                    } else {
                        log::info!("Disk full, dropped session {}", session_id);
                    }
                }
            }
        }
    }
}