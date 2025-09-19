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

pub mod store;
pub mod resource_tracker;
pub mod kv_cache;
pub mod kv_quantizer;

pub use store::DistributedObjectStore;
pub use resource_tracker::ResourceTracker;
pub use kv_cache::LogStructuredKVCache;
pub use kv_quantizer::KVQuantizer;


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_distributed_object_store() {
        let store = DistributedObjectStore::new();
        assert!(store.is_empty());
    }
}

