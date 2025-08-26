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

use std::sync::{Mutex, Arc};
use std::collections::HashSet;
use log;

//! Resource tracker for distributed object store instances
pub struct ResourceTracker {
    //! Set of distributed object store instances
    instances: Mutex<HashSet<Arc<DistributedObjectStore>>>,
}

impl ResourceTracker {
    pub fn new() -> Arc<Self> {
        //! Create a new resource tracker
        let tracker = Arc::new(ResourceTracker {
            //! Set of distributed object store instances
            instances: Mutex::new(HashSet::new()),
        });

        // Setup signal handlers (simplified in Rust)
        ctrlc::set_handler({
            let tracker = Arc::clone(&tracker);
            move || {
                log::info!("Received SIGINT, cleaning up resources");
                tracker.cleanup_all_resources();
                std::process::exit(0);
            }
        }).expect("Error setting Ctrl+C handler");

        // Register exit handler
        std::process::exit_hooks::add({
            let tracker = Arc::clone(&tracker);
            move || {
                tracker.cleanup_all_resources();
            }
        });

        tracker
    }

    pub fn register_instance(&self, instance: Arc<DistributedObjectStore>) {
        let mut instances = self.instances.lock().unwrap();
        instances.insert(instance);
    }

    pub fn unregister_instance(&self, instance: Arc<DistributedObjectStore>) {
        let mut instances = self.instances.lock().unwrap();
        instances.remove(&instance);
    }

    pub fn cleanup_all_resources(&self) {
        let instances = self.instances.lock().unwrap();
        for instance in instances.iter() {
            log::info!("Cleaning up DistributedObjectStore instance");
            instance.tear_down_all().await;
        }
    }
}