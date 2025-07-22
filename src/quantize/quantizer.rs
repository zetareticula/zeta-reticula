// Copyright 2025 ZETA RETICULA
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

use ndarray::{Array2, array};
use std::error::Error;
use p2pstore::KVCache;

pub struct Quantizer {
    bit_width: usize,
    scale_factor: f32,
}

impl Quantizer {
    pub fn new(bit_width: usize) -> Result<Self, Box<dyn Error>> {
        if bit_width < 1 || bit_width > 16 {
            return Err("Bit width must be between 1 and 16".into());
        }
        Ok(Quantizer {
            bit_width,
            scale_factor: 127.0 / f32::from(1 << (bit_width - 1)),
        })
    }

    pub fn quantize_kv_cache(&self, kv_cache: &mut KVCache) -> Result<(), Box<dyn Error>> {
        for buffer in &mut kv_cache.buffers {
            let mut data = Vec::with_capacity(buffer.size_ as usize);
            // Simulate float data (replace with actual data source)
            for _ in 0..buffer.size_ {
                data.push(rand::random::<f32>() * 10.0);
            }
            let array = Array2::from_shape_vec((buffer.size_ as usize, 1), data)?;
            let quantized = self.quantize_array(&array);
            buffer.size_ = quantized.len() as u64; // Update size to quantized data length
            // Here, you'd typically store quantized data back into buffer (simplified)
        }
        Ok(())
    }

    fn quantize_array(&self, array: &Array2<f32>) -> Vec<i8> {
        let mut quantized = Vec::with_capacity(array.len());
        for &val in array.iter() {
            let scaled = val * self.scale_factor;
            let clamped = scaled.clamp(-128.0, 127.0) as i8;
            quantized.push(clamped);
        }
        quantized
    }

    pub fn dequantize_kv_cache(&self, kv_cache: &mut KVCache) -> Result<(), Box<dyn Error>> {
        for buffer in &mut kv_cache.buffers {
            let mut data = Vec::with_capacity(buffer.size_ as usize);
            // Simulate quantized data (replace with actual data source)
            for _ in 0..buffer.size_ {
                data.push(rand::random::<i8>());
            }
            let array = Array2::from_shape_vec((buffer.size_ as usize, 1), data)?;
            let dequantized = self.dequantize_array(&array);
            buffer.size_ = dequantized.len() as u64; // Update size to dequantized data length
            // Here, you'd typically store dequantized data back into buffer (simplified)
        }
        Ok(())
    }

    fn dequantize_array(&self, array: &Array2<i8>) -> Vec<f32> {
        let mut dequantized = Vec::with_capacity(array.len());
        for &val in array.iter() {
            dequantized.push(val as f32 / self.scale_factor);
        }
        dequantized
    }
}