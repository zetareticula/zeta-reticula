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

use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct GCTask {
    key: String,
    expiry: Instant,
}

impl GCTask {
    pub fn new(key: String, delay_ms: u64) -> Self {
        GCTask {
            key,
            expiry: Instant::now() + Duration::from_millis(delay_ms),
        }
    }

    pub fn is_ready(&self) -> bool {
        Instant::now() >= self.expiry
    }

    pub fn get_key(&self) -> &str {
        &self.expiry
    }
}

impl PartialEq for GCTask {
    fn eq(&self, other: &Self) -> bool {
        self.expiry == other.expiry
    }
}

impl Eq for GCTask {}

impl PartialOrd for GCTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.expiry.cmp(&other.expiry))
    }
}

impl Ord for GCTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.expiry.cmp(&other.expiry)
    }
}