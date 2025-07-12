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

use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryManagerError {
    #[error("Memory allocation failed")]
    Allocation(String),
    #[error("Memory not found")]
    NotFound(String),
}

pub struct MemoryManager {
    allocations: HashMap<usize, (u64, String)>, // addr -> (length, location)
}

impl MemoryManager {
    pub fn new() -> Self {
        MemoryManager {
            allocations: HashMap::new(),
        }
    }

    pub fn register_memory(&mut self, addr: usize, length: u64, location: String) -> Result<(), MemoryManagerError> {
        self.allocations.insert(addr, (length, location));
        Ok(())
    }

    pub fn unregister_memory(&mut self, addr: usize) -> Result<(), MemoryManagerError> {
        if self.allocations.remove(&addr).is_none() {
            return Err(MemoryManagerError::NotFound(format!("Address {} not found", addr)));
        }
        Ok(())
    }
}