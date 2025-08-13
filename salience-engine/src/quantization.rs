// Copyright 2025 ZETA RETICULA INC
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


use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantizationResult {
    pub row: usize,
    pub col: usize,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrecisionLevel {
    Bit4,
    Bit8,
    Bit16,
    Bit32,
}

impl Default for PrecisionLevel {
    fn default() -> Self {
        Self::Bit32
    }
}
