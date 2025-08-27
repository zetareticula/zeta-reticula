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


use serde::{Serialize, Deserialize};

/// Represents the result of quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationResult {
    /// The original value before quantization
    pub original: f32,
    /// The quantized value
    pub quantized: f32,
    /// The scale factor used for quantization
    pub scale: f32,
    /// The zero point used for quantization (for asymmetric quantization)
    pub zero_point: Option<i32>,
    /// The precision level used for quantization
    pub precision: PrecisionLevel,
}

/// Represents different precision levels for quantization
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PrecisionLevel {
    /// 4-bit quantization
    Bit4,
    /// 8-bit quantization
    Bit8,
    /// 16-bit quantization
    Bit16,
    /// 32-bit quantization (full precision)
    Bit32,
    /// Custom bit width
    Custom(u8),
}

impl Default for PrecisionLevel {
    fn default() -> Self {
        PrecisionLevel::Bit32
    }
}

impl Default for QuantizationResult {
    fn default() -> Self {
        Self {
            original: 0.0,
            quantized: 0.0,
            scale: 1.0,
            zero_point: None,
            precision: PrecisionLevel::Bit32,
        }
    }
}

impl QuantizationResult {
    /// Creates a new QuantizationResult
    pub fn new(
        original: f32,
        quantized: f32,
        scale: f32,
        zero_point: Option<i32>,
        precision: PrecisionLevel,
    ) -> Self {
        Self {
            original,
            quantized,
            scale,
            zero_point,
            precision,
        }
    }

    /// Calculates the quantization error
    pub fn error(&self) -> f32 {
        (self.original - self.quantized).abs()
    }

    /// Calculates the relative error
    pub fn relative_error(&self) -> f32 {
        if self.original == 0.0 {
            0.0
        } else {
            self.error() / self.original.abs()
        }
    }
}
