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

//! # KVQuantizer - Production-Ready Quantization Engine
//!
//! This module provides high-performance, memory-efficient quantization for LLM inference.
//! It implements symmetric min-max quantization with support for Int8, Int4, Int2, and Bit1
//! precision levels.
//!
//! ## Design Principles
//!
//! - **Zero-copy where possible**: Uses slices and references to avoid unnecessary allocations
//! - **Ownership-based memory management**: No garbage collection, deterministic cleanup
//! - **Cache-friendly**: Processes data in contiguous blocks for optimal CPU cache utilization
//! - **Thread-safe**: All shared state protected by `Arc` for safe concurrent access
//!
//! ## Quantization Algorithm
//!
//! Uses symmetric min-max quantization:
//! - `scale = max(|min|, |max|) / (2^(bits-1) - 1)`
//! - `quantized = round(value / scale)`
//! - `dequantized = quantized * scale`

use crate::{
    KVQuantConfig, PrecisionLevel, QuantizationResult, RoleInferer, MesolimbicSystem,
};
use crate::block::DataBlock;
use crate::kvquant_config::QuantizationError;
use dashmap::DashMap;
use std::sync::Arc;

/// Quantization parameters computed from input data.
/// Stored alongside quantized data for accurate dequantization.
#[derive(Debug, Clone, Copy)]
pub struct QuantizationParams {
    /// Scale factor for converting between float and quantized values
    pub scale: f32,
    /// Zero point offset (0 for symmetric quantization)
    pub zero_point: i32,
    /// Number of bits used for quantization
    pub bits: u8,
}

impl QuantizationParams {
    /// Compute quantization parameters from input data using symmetric min-max scaling.
    ///
    /// # Arguments
    /// * `input` - Slice of f32 values to analyze
    /// * `bits` - Target bit width (1, 2, 4, or 8)
    ///
    /// # Returns
    /// Quantization parameters optimized for the input distribution
    #[inline]
    pub fn from_data(input: &[f32], bits: u8) -> Self {
        if input.is_empty() {
            return Self { scale: 1.0, zero_point: 0, bits };
        }

        // Find absolute max for symmetric quantization
        let abs_max = input.iter()
            .map(|&x| x.abs())
            .fold(0.0f32, f32::max);

        // Compute scale: maps [-abs_max, abs_max] to quantized range
        let qmax = ((1u32 << (bits - 1)) - 1) as f32;
        let scale = if abs_max > 0.0 { abs_max / qmax } else { 1.0 };

        Self { scale, zero_point: 0, bits }
    }
}

/// Main KVQuantizer struct that handles the quantization process.
///
/// # Thread Safety
/// 
/// `KVQuantizer` is `Send + Sync` and can be safely shared across threads.
/// Internal state is protected by `DashMap` (concurrent hashmap) and `Arc`.
///
/// # Memory Management
///
/// Uses Rust's ownership system for deterministic memory management:
/// - No garbage collection overhead
/// - Predictable memory usage
/// - Zero-copy operations where possible
#[derive(Debug, Clone)]
pub struct KVQuantizer {
    /// Configuration for the quantizer
    pub config: KVQuantConfig,
    /// Data blocks for quantization (thread-safe concurrent map)
    pub data_blocks: DashMap<usize, DataBlock>,
    /// Role inference component (shared ownership via Arc)
    pub role_inferer: Arc<RoleInferer>,
    /// Mesolimbic system for reinforcement learning (shared ownership via Arc)
    pub mesolimbic_system: Arc<MesolimbicSystem>,
}

impl PartialEq for KVQuantizer {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
            && self.role_inferer.threshold == other.role_inferer.threshold
            && self.mesolimbic_system.learning_rate == other.mesolimbic_system.learning_rate
            && self.mesolimbic_system.discount_factor == other.mesolimbic_system.discount_factor
    }
}

impl Eq for KVQuantizer {}

impl KVQuantizer {
    /// Create a new KVQuantizer with the given configuration.
    ///
    /// # Arguments
    /// * `config` - Configuration specifying precision, block size, and other parameters
    ///
    /// # Example
    /// ```
    /// use kvquant_rs::{KVQuantizer, KVQuantConfig, PrecisionLevel};
    /// 
    /// let config = KVQuantConfig {
    ///     precision: PrecisionLevel::Int8,
    ///     ..Default::default()
    /// };
    /// let quantizer = KVQuantizer::new(config);
    /// ```
    pub fn new(config: KVQuantConfig) -> Self {
        Self {
            config: config.clone(),
            data_blocks: DashMap::new(),
            role_inferer: Arc::new(RoleInferer::default()),
            mesolimbic_system: Arc::new(MesolimbicSystem::default()),
        }
    }

    /// Quantize input data to the configured precision level.
    ///
    /// # Arguments
    /// * `input` - Slice of f32 values to quantize
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Quantized data with scale factor prepended (first 4 bytes)
    /// * `Err(QuantizationError)` - If quantization fails
    ///
    /// # Memory Layout
    /// Output format: `[scale: f32 (4 bytes)][quantized_data: u8...]`
    ///
    /// # Example
    /// ```
    /// use kvquant_rs::{KVQuantizer, KVQuantConfig};
    /// 
    /// let quantizer = KVQuantizer::new(KVQuantConfig::default());
    /// let input = vec![0.5, -0.3, 0.8, -0.1];
    /// let quantized = quantizer.quantize(&input).unwrap();
    /// ```
    pub fn quantize(&self, input: &[f32]) -> QuantizationResult<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        match self.config.precision {
            PrecisionLevel::Int8 => self.quantize_int8(input),
            PrecisionLevel::Int4 => self.quantize_int4(input),
            PrecisionLevel::Int2 => self.quantize_int2(input),
            PrecisionLevel::Bit1 => self.quantize_bit1(input),
        }
    }

    /// Dequantize data back to f32 values.
    ///
    /// # Arguments
    /// * `quantized` - Quantized data with scale factor prepended
    ///
    /// # Returns
    /// * `Ok(Vec<f32>)` - Reconstructed float values
    /// * `Err(QuantizationError)` - If dequantization fails
    pub fn dequantize(&self, quantized: &[u8]) -> QuantizationResult<Vec<f32>> {
        if quantized.is_empty() {
            return Ok(Vec::new());
        }

        match self.config.precision {
            PrecisionLevel::Int8 => self.dequantize_int8(quantized),
            PrecisionLevel::Int4 => self.dequantize_int4(quantized),
            PrecisionLevel::Int2 => self.dequantize_int2(quantized),
            PrecisionLevel::Bit1 => self.dequantize_bit1(quantized),
        }
    }

    /// 8-bit symmetric quantization.
    /// Maps f32 values to i8 range [-127, 127] using computed scale factor.
    #[inline]
    fn quantize_int8(&self, input: &[f32]) -> QuantizationResult<Vec<u8>> {
        let params = QuantizationParams::from_data(input, 8);
        let inv_scale = 1.0 / params.scale;

        // Pre-allocate: 4 bytes for scale + 1 byte per value
        let mut output = Vec::with_capacity(4 + input.len());
        
        // Store scale factor as first 4 bytes for dequantization
        output.extend_from_slice(&params.scale.to_le_bytes());

        // Quantize each value: clamp to [-127, 127], store as u8
        for &val in input {
            let quantized = (val * inv_scale).round().clamp(-127.0, 127.0) as i8;
            output.push(quantized as u8);
        }

        Ok(output)
    }

    /// 4-bit symmetric quantization.
    /// Packs two 4-bit values into each byte for 2x compression over Int8.
    #[inline]
    fn quantize_int4(&self, input: &[f32]) -> QuantizationResult<Vec<u8>> {
        let params = QuantizationParams::from_data(input, 4);
        let inv_scale = 1.0 / params.scale;
        let qmax = 7.0; // 4-bit signed: [-7, 7]

        // Pre-allocate: 4 bytes scale + ceil(input.len() / 2) bytes for packed data
        let packed_len = (input.len() + 1) / 2;
        let mut output = Vec::with_capacity(4 + packed_len);
        
        output.extend_from_slice(&params.scale.to_le_bytes());

        // Pack two 4-bit values per byte
        let mut i = 0;
        while i < input.len() {
            let low = ((input[i] * inv_scale).round().clamp(-qmax, qmax) as i8 + 8) as u8 & 0x0F;
            let high = if i + 1 < input.len() {
                ((input[i + 1] * inv_scale).round().clamp(-qmax, qmax) as i8 + 8) as u8 & 0x0F
            } else {
                8 // Zero padding for odd-length input
            };
            output.push((high << 4) | low);
            i += 2;
        }

        Ok(output)
    }

    /// 2-bit symmetric quantization.
    /// Packs four 2-bit values into each byte for 4x compression over Int8.
    #[inline]
    fn quantize_int2(&self, input: &[f32]) -> QuantizationResult<Vec<u8>> {
        let params = QuantizationParams::from_data(input, 2);
        let inv_scale = 1.0 / params.scale;
        let qmax = 1.0; // 2-bit signed: [-1, 1]

        // Pre-allocate: 4 bytes scale + ceil(input.len() / 4) bytes
        let packed_len = (input.len() + 3) / 4;
        let mut output = Vec::with_capacity(4 + packed_len);
        
        output.extend_from_slice(&params.scale.to_le_bytes());

        // Pack four 2-bit values per byte
        let mut i = 0;
        while i < input.len() {
            let mut byte: u8 = 0;
            for j in 0..4 {
                let val = if i + j < input.len() {
                    ((input[i + j] * inv_scale).round().clamp(-qmax, qmax) as i8 + 2) as u8 & 0x03
                } else {
                    2 // Zero padding
                };
                byte |= val << (j * 2);
            }
            output.push(byte);
            i += 4;
        }

        Ok(output)
    }

    /// 1-bit quantization (binary).
    /// Each value becomes 0 (negative) or 1 (non-negative), packed 8 per byte.
    #[inline]
    fn quantize_bit1(&self, input: &[f32]) -> QuantizationResult<Vec<u8>> {
        let params = QuantizationParams::from_data(input, 1);

        // Pre-allocate: 4 bytes scale + ceil(input.len() / 8) bytes
        let packed_len = (input.len() + 7) / 8;
        let mut output = Vec::with_capacity(4 + packed_len);
        
        output.extend_from_slice(&params.scale.to_le_bytes());

        // Pack 8 bits per byte
        let mut i = 0;
        while i < input.len() {
            let mut byte: u8 = 0;
            for j in 0..8 {
                if i + j < input.len() && input[i + j] >= 0.0 {
                    byte |= 1 << j;
                }
            }
            output.push(byte);
            i += 8;
        }

        Ok(output)
    }

    /// Dequantize 8-bit data back to f32.
    #[inline]
    fn dequantize_int8(&self, quantized: &[u8]) -> QuantizationResult<Vec<f32>> {
        if quantized.len() < 4 {
            return Err(QuantizationError::InvalidInput(
                "Quantized data too short: missing scale factor".into()
            ));
        }

        // Extract scale from first 4 bytes
        let scale = f32::from_le_bytes([quantized[0], quantized[1], quantized[2], quantized[3]]);
        
        // Dequantize remaining bytes
        let mut output = Vec::with_capacity(quantized.len() - 4);
        for &byte in &quantized[4..] {
            let quantized_val = byte as i8;
            output.push(quantized_val as f32 * scale);
        }

        Ok(output)
    }

    /// Dequantize 4-bit packed data back to f32.
    #[inline]
    fn dequantize_int4(&self, quantized: &[u8]) -> QuantizationResult<Vec<f32>> {
        if quantized.len() < 4 {
            return Err(QuantizationError::InvalidInput(
                "Quantized data too short: missing scale factor".into()
            ));
        }

        let scale = f32::from_le_bytes([quantized[0], quantized[1], quantized[2], quantized[3]]);
        
        // Each byte contains two 4-bit values
        let mut output = Vec::with_capacity((quantized.len() - 4) * 2);
        for &byte in &quantized[4..] {
            let low = ((byte & 0x0F) as i8 - 8) as f32 * scale;
            let high = ((byte >> 4) as i8 - 8) as f32 * scale;
            output.push(low);
            output.push(high);
        }

        Ok(output)
    }

    /// Dequantize 2-bit packed data back to f32.
    #[inline]
    fn dequantize_int2(&self, quantized: &[u8]) -> QuantizationResult<Vec<f32>> {
        if quantized.len() < 4 {
            return Err(QuantizationError::InvalidInput(
                "Quantized data too short: missing scale factor".into()
            ));
        }

        let scale = f32::from_le_bytes([quantized[0], quantized[1], quantized[2], quantized[3]]);
        
        // Each byte contains four 2-bit values
        let mut output = Vec::with_capacity((quantized.len() - 4) * 4);
        for &byte in &quantized[4..] {
            for j in 0..4 {
                let val = ((byte >> (j * 2)) & 0x03) as i8 - 2;
                output.push(val as f32 * scale);
            }
        }

        Ok(output)
    }

    /// Dequantize 1-bit packed data back to f32.
    #[inline]
    fn dequantize_bit1(&self, quantized: &[u8]) -> QuantizationResult<Vec<f32>> {
        if quantized.len() < 4 {
            return Err(QuantizationError::InvalidInput(
                "Quantized data too short: missing scale factor".into()
            ));
        }

        let scale = f32::from_le_bytes([quantized[0], quantized[1], quantized[2], quantized[3]]);
        
        // Each byte contains 8 bits
        let mut output = Vec::with_capacity((quantized.len() - 4) * 8);
        for &byte in &quantized[4..] {
            for j in 0..8 {
                let bit = (byte >> j) & 1;
                // Map: 0 -> -scale, 1 -> +scale
                output.push(if bit == 1 { scale } else { -scale });
            }
        }

        Ok(output)
    }

    /// Get compression ratio for current precision level.
    ///
    /// # Returns
    /// Ratio of original size to compressed size (e.g., 4.0 for Int8 means 4x smaller)
    pub fn compression_ratio(&self) -> f32 {
        match self.config.precision {
            PrecisionLevel::Int8 => 4.0,  // f32 (4 bytes) -> i8 (1 byte)
            PrecisionLevel::Int4 => 8.0,  // f32 (4 bytes) -> 4 bits (0.5 bytes)
            PrecisionLevel::Int2 => 16.0, // f32 (4 bytes) -> 2 bits (0.25 bytes)
            PrecisionLevel::Bit1 => 32.0, // f32 (4 bytes) -> 1 bit (0.125 bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quantize_dequantize_int8() {
        let config = KVQuantConfig {
            block_size: 1024,
            spot_capacity: 2,
            salience_threshold: 0.5,
            precision: PrecisionLevel::Int8,
            enable_debug_logging: false,
            max_cache_items: 1000,
        };
        
        let quantizer = KVQuantizer::new(config);
        let input = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        
        let quantized = quantizer.quantize(&input).unwrap();
        let dequantized = quantizer.dequantize(&quantized).unwrap();
        
        // Check if dequantized values are close to original (with some loss)
        for (orig, deq) in input.iter().zip(dequantized) {
            assert!((orig - deq).abs() < 0.01, "Original: {}, Dequantized: {}", orig, deq);
        }
    }
}
