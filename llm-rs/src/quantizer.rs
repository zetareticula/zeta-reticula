//! Adaptive quantization module for KV cache values

use std::sync::Arc;
use std::collections::HashMap;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};
use ndarray::{Array2, ArrayView2};
use rand::Rng;

#[derive(Error, Debug)]
pub enum QuantizationError {
    #[error("Invalid bit depth: {0}")]
    InvalidBitDepth(u8),
    #[error("Quantization failed: {0}")]
    QuantizationFailed(String),
    #[error("Dequantization failed: {0}")]
    DequantizationFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationConfig {
    pub min_bit_depth: u8,
    pub max_bit_depth: u8,
    pub initial_bit_depth: u8,
    pub compression_threshold: f32,
}

impl Default for QuantizationConfig {
    fn default() -> Self {
        Self {
            min_bit_depth: 2,
            max_bit_depth: 8,
            initial_bit_depth: 8,
            compression_threshold: 0.5,
        }
    }
}

pub struct Quantizer {
    config: QuantizationConfig,
    // Cache for quantization parameters to avoid recomputation
    quantization_params: parking_lot::RwLock<HashMap<u8, (f32, f32)>>, // (scale, zero_point)
}

impl Quantizer {
    pub fn new(config: QuantizationConfig) -> Self {
        // Validate config
        if config.min_bit_depth < 1 || config.max_bit_depth > 8 || config.min_bit_depth > config.max_bit_depth {
            panic!("Invalid quantization configuration");
        }
        
        if config.initial_bit_depth < config.min_bit_depth || config.initial_bit_depth > config.max_bit_depth {
            panic!("Initial bit depth must be between min and max bit depth");
        }
        
        Self {
            config,
            quantization_params: parking_lot::RwLock::new(HashMap::new()),
        }
    }
    
    pub fn quantize(&self, data: &[u8], bit_depth: u8) -> Result<Vec<u8>, QuantizationError> {
        self.validate_bit_depth(bit_depth)?;
        
        if bit_depth == 8 {
            return Ok(data.to_vec());
        }
        
        let (scale, zero_point) = self.compute_quantization_params(data, bit_depth);
        
        // Quantize the data
        let max_val = (1u16 << bit_depth) - 1;
        let scale_recip = 1.0 / scale;
        
        let quantized: Vec<u8> = data
            .iter()
            .map(|&x| {
                let x_float = x as f32;
                let q = ((x_float * scale_recip) + zero_point).round() as u16;
                q.min(max_val) as u8
            })
            .collect();
            
        Ok(quantized)
    }
    
    pub fn dequantize(&self, data: &[u8], bit_depth: u8) -> Result<Vec<u8>, QuantizationError> {
        self.validate_bit_depth(bit_depth)?;
        
        if bit_depth == 8 {
            return Ok(data.to_vec());
        }
        
        // Get or compute quantization params
        let (scale, zero_point) = if let Some(params) = self.quantization_params.read().get(&bit_depth) {
            *params
        } else {
            // If we don't have params (shouldn't happen in normal operation),
            // use default params that won't cause division by zero
            (1.0, 0.0)
        };
        
        // Dequantize the data
        let dequantized: Vec<u8> = data
            .iter()
            .map(|&q| {
                let q_float = q as f32;
                let x = (q_float - zero_point) * scale;
                x.round().max(0.0).min(255.0) as u8
            })
            .collect();
            
        Ok(dequantized)
    }
    
    pub fn quantize_2d(&self, data: &Array2<f32>, bit_depth: u8) -> Result<(Array2<u8>, f32, f32), QuantizationError> {
        self.validate_bit_depth(bit_depth)?;
        
        if bit_depth == 32 {
            return Err(QuantizationError::InvalidBitDepth(bit_depth));
        }
        
        let (scale, zero_point) = self.compute_quantization_params_2d(data, bit_depth);
        let scale_recip = 1.0 / scale;
        let max_val = (1u16 << bit_depth) - 1;
        
        let mut quantized = Array2::<u8>::zeros(data.dim());
        
        for ((i, j), &val) in data.indexed_iter() {
            let q = ((val * scale_recip) + zero_point).round() as u16;
            quantized[(i, j)] = q.min(max_val) as u8;
        }
        
        Ok((quantized, scale, zero_point))
    }
    
    pub fn dequantize_2d(&self, data: &Array2<u8>, bit_depth: u8, scale: f32, zero_point: f32) -> Result<Array2<f32>, QuantizationError> {
        self.validate_bit_depth(bit_depth)?;
        
        let mut dequantized = Array2::<f32>::zeros(data.dim());
        
        for ((i, j), &val) in data.indexed_iter() {
            dequantized[(i, j)] = (val as f32 - zero_point) * scale;
        }
        
        Ok(dequantized)
    }
    
    fn compute_quantization_params(&self, data: &[u8], bit_depth: u8) -> (f32, f32) {
        // Check if we already computed these params
        if let Some(params) = self.quantization_params.read().get(&bit_depth) {
            return *params;
        }
        
        // Compute min and max
        let min_val = *data.iter().min().unwrap_or(&0) as f32;
        let max_val = *data.iter().max().unwrap_or(&255) as f32;
        
        self.compute_params_from_min_max(min_val, max_val, bit_depth)
    }
    
    fn compute_quantization_params_2d(&self, data: &Array2<f32>, bit_depth: u8) -> (f32, f32) {
        // For 2D arrays, we might want to use a more sophisticated approach
        // like per-channel quantization, but for now we'll just use min/max
        let min_val = data.fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = data.fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        self.compute_params_from_min_max(min_val, max_val, bit_depth)
    }
    
    fn compute_params_from_min_max(&self, min_val: f32, max_val: f32, bit_depth: u8) -> (f32, f32) {
        let qmin = 0.0;
        let qmax = (1u16 << bit_depth) as f32 - 1.0;
        
        // Ensure we don't have a single value
        let min_val = min_val.min(max_val - 0.01);
        
        let scale = (max_val - min_val) / (qmax - qmin);
        let zero_point = qmin - min_val / scale;
        
        let zero_point = if zero_point < qmin {
            qmin
        } else if zero_point > qmax {
            qmax
        } else {
            zero_point.round()
        };
        
        let params = (scale, zero_point);
        
        // Cache the computed params
        self.quantization_params.write().insert(bit_depth, params);
        
        params
    }
    
    fn validate_bit_depth(&self, bit_depth: u8) -> Result<(), QuantizationError> {
        if bit_depth < self.config.min_bit_depth || bit_depth > self.config.max_bit_depth {
            return Err(QuantizationError::InvalidBitDepth(bit_depth));
        }
        Ok(())
    }
    
    pub fn should_compress(&self, original_size: usize, compressed_size: usize) -> bool {
        let ratio = compressed_size as f32 / original_size as f32;
        ratio <= self.config.compression_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;
    
    #[test]
    fn test_quantization_roundtrip() {
        let config = QuantizationConfig {
            min_bit_depth: 2,
            max_bit_depth: 8,
            initial_bit_depth: 8,
            compression_threshold: 0.5,
        };
        
        let quantizer = Quantizer::new(config);
        let data: Vec<u8> = (0..256).map(|x| x as u8).collect();
        
        // Test different bit depths
        for &bit_depth in &[2, 4, 6, 8] {
            let quantized = quantizer.quantize(&data, bit_depth).unwrap();
            let dequantized = quantizer.dequantize(&quantized, bit_depth).unwrap();
            
            // For lower bit depths, we expect some loss
            if bit_depth < 8 {
                assert_ne!(data, dequantized);
            } else {
                assert_eq!(data, dequantized);
            }
            
            // Check that the values are in the expected range
            assert!(quantized.iter().all(|&x| x < (1 << bit_depth)));
        }
    }
    
    #[test]
    fn test_2d_quantization() {
        let config = QuantizationConfig::default();
        let quantizer = Quantizer::new(config);
        
        let data = array![
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [7.0, 8.0, 9.0]
        ];
        
        let (quantized, scale, zero_point) = quantizer.quantize_2d(&data, 4).unwrap();
        let dequantized = quantizer.dequantize_2d(&quantized, 4, scale, zero_point).unwrap();
        
        // Check that the dequantized values are close to the original
        for (orig, deq) in data.iter().zip(dequantized.iter()) {
            assert!((orig - deq).abs() < 0.1);
        }
    }
    
    #[test]
    fn test_compression_decision() {
        let mut config = QuantizationConfig::default();
        config.compression_threshold = 0.5;
        
        let quantizer = Quantizer::new(config);
        
        // Should compress (0.4 < 0.5)
        assert!(quantizer.should_compress(100, 40));
        
        // Should not compress (0.6 > 0.5)
        assert!(!quantizer.should_compress(100, 60));
    }
}
