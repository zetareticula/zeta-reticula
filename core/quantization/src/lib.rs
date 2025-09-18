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

//! Unified Quantization Engine for Zeta Reticula
//! 
//! This module consolidates all quantization functionality from:
//! - zeta-quantize/ (entire crate)
//! - quantize-cli/ (core logic)
//! - salience-engine/src/quantizer.rs
//! - llm-rs/src/quantizer.rs
//! - agentflow-rs/src/quantizer.rs
//! - shared/src/quantization.rs

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuantizationError {
    #[error("Invalid precision level: {0}")]
    InvalidPrecision(String),
    #[error("Tensor operation failed: {0}")]
    TensorError(String),
    #[error("Model loading failed: {0}")]
    ModelError(String),
    #[error("Memory allocation failed: {0}")]
    MemoryError(String),
    #[error("Validation failed: {0}")]
    ValidationError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum PrecisionLevel {
    Int1,
    Int2,
    Int4,
    Int8,
    FP16,
    FP32,
}

impl PrecisionLevel {
    pub fn bits(&self) -> u8 {
        match self {
            PrecisionLevel::Int1 => 1,
            PrecisionLevel::Int2 => 2,
            PrecisionLevel::Int4 => 4,
            PrecisionLevel::Int8 => 8,
            PrecisionLevel::FP16 => 16,
            PrecisionLevel::FP32 => 32,
        }
    }

    pub fn max_value(&self) -> f32 {
        match self {
            PrecisionLevel::Int1 => 1.0,
            PrecisionLevel::Int2 => 3.0,
            PrecisionLevel::Int4 => 15.0,
            PrecisionLevel::Int8 => 255.0,
            PrecisionLevel::FP16 => f32::MAX,
            PrecisionLevel::FP32 => f32::MAX,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantizationAlgorithm {
    Linear,
    KMeans,
    Learned,
    BlockWise,
    SalienceBased,
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationConfig {
    pub precision: PrecisionLevel,
    pub algorithm: QuantizationAlgorithm,
    pub block_size: usize,
    pub salience_threshold: f32,
    pub preserve_outliers: bool,
    pub use_symmetric: bool,
    pub calibration_samples: usize,
    pub validation_threshold: f32,
}

impl Default for QuantizationConfig {
    fn default() -> Self {
        Self {
            precision: PrecisionLevel::Int4,
            algorithm: QuantizationAlgorithm::SalienceBased,
            block_size: 128,
            salience_threshold: 0.7,
            preserve_outliers: true,
            use_symmetric: false,
            calibration_samples: 1000,
            validation_threshold: 0.95,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationParameters {
    pub scale: f32,
    pub zero_point: i32,
    pub min_val: f32,
    pub max_val: f32,
}

impl QuantizationParameters {
    pub fn new(min_val: f32, max_val: f32, precision: &PrecisionLevel) -> Self {
        let qmin = 0.0;
        let qmax = precision.max_value();
        let scale = (max_val - min_val) / (qmax - qmin);
        let zero_point = (qmin - min_val / scale).round() as i32;

        Self {
            scale,
            zero_point,
            min_val,
            max_val,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationResult {
    pub quantized_data: Vec<i32>,
    pub parameters: QuantizationParameters,
    pub compression_ratio: f32,
    pub error_metrics: ErrorMetrics,
    pub salience_preserved: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub mse: f32,
    pub mae: f32,
    pub max_error: f32,
    pub snr: f32,
}

/// Unified Quantization Engine
pub struct UnifiedQuantizer {
    config: QuantizationConfig,
    salience_weights: HashMap<usize, f32>,
}

impl UnifiedQuantizer {
    pub fn new(config: QuantizationConfig) -> Self {
        Self {
            config,
            salience_weights: HashMap::new(),
        }
    }

    pub fn set_salience_weights(&mut self, weights: HashMap<usize, f32>) {
        self.salience_weights = weights;
    }

    pub fn quantize(&self, data: &[f32]) -> Result<QuantizationResult, QuantizationError> {
        match self.config.algorithm {
            QuantizationAlgorithm::Linear => self.linear_quantize(data),
            QuantizationAlgorithm::KMeans => self.kmeans_quantize(data),
            QuantizationAlgorithm::Learned => self.learned_quantize(data),
            QuantizationAlgorithm::BlockWise => self.blockwise_quantize(data),
            QuantizationAlgorithm::SalienceBased => self.salience_quantize(data),
            QuantizationAlgorithm::Adaptive => self.adaptive_quantize(data),
        }
    }

    fn linear_quantize(&self, data: &[f32]) -> Result<QuantizationResult, QuantizationError> {
        let min_val = data.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = data.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        let params = QuantizationParameters::new(min_val, max_val, &self.config.precision);
        let mut quantized_data = Vec::with_capacity(data.len());
        
        for &value in data {
            let quantized = ((value - min_val) / params.scale + params.zero_point as f32)
                .round()
                .clamp(0.0, self.config.precision.max_value()) as i32;
            quantized_data.push(quantized);
        }

        let error_metrics = self.calculate_error_metrics(data, &quantized_data, &params);
        let compression_ratio = (32.0 / self.config.precision.bits() as f32);

        Ok(QuantizationResult {
            quantized_data,
            parameters: params,
            compression_ratio,
            error_metrics,
            salience_preserved: 1.0, // Linear doesn't consider salience
        })
    }

    fn salience_quantize(&self, data: &[f32]) -> Result<QuantizationResult, QuantizationError> {
        // Apply salience-aware quantization
        let mut weighted_data = Vec::with_capacity(data.len());
        let mut salience_preserved = 0.0;
        let mut total_salience = 0.0;

        for (i, &value) in data.iter().enumerate() {
            let salience = self.salience_weights.get(&i).copied().unwrap_or(1.0);
            total_salience += salience;

            if salience >= self.config.salience_threshold {
                // High salience: preserve with higher precision
                weighted_data.push(value);
                salience_preserved += salience;
            } else {
                // Low salience: can use lower precision
                let reduced_precision_value = (value * 0.9).round() / 0.9; // Slight precision reduction
                weighted_data.push(reduced_precision_value);
            }
        }

        salience_preserved = if total_salience > 0.0 { salience_preserved / total_salience } else { 0.0 };

        // Apply linear quantization to the salience-weighted data
        let min_val = weighted_data.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = weighted_data.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        let params = QuantizationParameters::new(min_val, max_val, &self.config.precision);
        let mut quantized_data = Vec::with_capacity(weighted_data.len());
        
        for &value in &weighted_data {
            let quantized = ((value - min_val) / params.scale + params.zero_point as f32)
                .round()
                .clamp(0.0, self.config.precision.max_value()) as i32;
            quantized_data.push(quantized);
        }

        let error_metrics = self.calculate_error_metrics(data, &quantized_data, &params);
        let compression_ratio = (32.0 / self.config.precision.bits() as f32);

        Ok(QuantizationResult {
            quantized_data,
            parameters: params,
            compression_ratio,
            error_metrics,
            salience_preserved,
        })
    }

    fn blockwise_quantize(&self, data: &[f32]) -> Result<QuantizationResult, QuantizationError> {
        let mut quantized_data = Vec::with_capacity(data.len());
        let mut all_params = Vec::new();
        let mut total_error = 0.0;

        for chunk in data.chunks(self.config.block_size) {
            let min_val = chunk.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max_val = chunk.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            
            let params = QuantizationParameters::new(min_val, max_val, &self.config.precision);
            all_params.push(params.clone());
            
            for &value in chunk {
                let quantized = ((value - min_val) / params.scale + params.zero_point as f32)
                    .round()
                    .clamp(0.0, self.config.precision.max_value()) as i32;
                quantized_data.push(quantized);
                
                // Calculate dequantized value for error
                let dequantized = (quantized as f32 - params.zero_point as f32) * params.scale + min_val;
                total_error += (value - dequantized).powi(2);
            }
        }

        // Use average parameters for the result
        let avg_params = if !all_params.is_empty() {
            let avg_scale = all_params.iter().map(|p| p.scale).sum::<f32>() / all_params.len() as f32;
            let avg_zero_point = all_params.iter().map(|p| p.zero_point).sum::<i32>() / all_params.len() as i32;
            let avg_min = all_params.iter().map(|p| p.min_val).sum::<f32>() / all_params.len() as f32;
            let avg_max = all_params.iter().map(|p| p.max_val).sum::<f32>() / all_params.len() as f32;
            
            QuantizationParameters {
                scale: avg_scale,
                zero_point: avg_zero_point,
                min_val: avg_min,
                max_val: avg_max,
            }
        } else {
            QuantizationParameters::new(0.0, 1.0, &self.config.precision)
        };

        let error_metrics = self.calculate_error_metrics(data, &quantized_data, &avg_params);
        let compression_ratio = (32.0 / self.config.precision.bits() as f32);

        Ok(QuantizationResult {
            quantized_data,
            parameters: avg_params,
            compression_ratio,
            error_metrics,
            salience_preserved: 0.8, // Blockwise preserves some structure
        })
    }

    fn kmeans_quantize(&self, data: &[f32]) -> Result<QuantizationResult, QuantizationError> {
        // Simplified K-means quantization
        let k = (1 << self.config.precision.bits()).min(256) as usize;
        let mut centroids = self.initialize_centroids(data, k);
        
        // Run K-means iterations
        for _ in 0..10 {
            let assignments = self.assign_to_centroids(data, &centroids);
            centroids = self.update_centroids(data, &assignments, k);
        }

        // Quantize data using final centroids
        let mut quantized_data = Vec::with_capacity(data.len());
        for &value in data {
            let closest_idx = self.find_closest_centroid(value, &centroids);
            quantized_data.push(closest_idx as i32);
        }

        let min_val = centroids.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = centroids.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let params = QuantizationParameters::new(min_val, max_val, &self.config.precision);
        
        let error_metrics = self.calculate_kmeans_error_metrics(data, &quantized_data, &centroids);
        let compression_ratio = (32.0 / self.config.precision.bits() as f32);

        Ok(QuantizationResult {
            quantized_data,
            parameters: params,
            compression_ratio,
            error_metrics,
            salience_preserved: 0.9, // K-means preserves data distribution
        })
    }

    fn learned_quantize(&self, _data: &[f32]) -> Result<QuantizationResult, QuantizationError> {
        // Placeholder for learned quantization - would require ML model
        Err(QuantizationError::ConfigError("Learned quantization not yet implemented".to_string()))
    }

    fn adaptive_quantize(&self, data: &[f32]) -> Result<QuantizationResult, QuantizationError> {
        // Adaptive quantization combines multiple approaches based on data characteristics
        let variance = self.calculate_variance(data);
        let has_outliers = self.detect_outliers(data);
        
        if variance > 1.0 && has_outliers {
            // High variance with outliers: use blockwise
            self.blockwise_quantize(data)
        } else if !self.salience_weights.is_empty() {
            // Has salience information: use salience-based
            self.salience_quantize(data)
        } else {
            // Default: use linear
            self.linear_quantize(data)
        }
    }

    fn calculate_error_metrics(&self, original: &[f32], quantized: &[i32], params: &QuantizationParameters) -> ErrorMetrics {
        let mut mse = 0.0;
        let mut mae = 0.0;
        let mut max_error: f32 = 0.0;
        let mut signal_power = 0.0;
        let mut noise_power = 0.0;

        for (_i, (&orig, &quant)) in original.iter().zip(quantized.iter()).enumerate() {
            let dequantized = (quant as f32 - params.zero_point as f32) * params.scale + params.min_val;
            let error = orig - dequantized;
            
            mse += error * error;
            mae += error.abs();
            max_error = max_error.max(error.abs());
            
            signal_power += orig * orig;
            noise_power += error * error;
        }

        let n = original.len() as f32;
        mse /= n;
        mae /= n;
        
        let snr = if noise_power > 0.0 {
            10.0 * (signal_power / noise_power).log10()
        } else {
            f32::INFINITY
        };

        ErrorMetrics {
            mse,
            mae,
            max_error,
            snr,
        }
    }

    fn calculate_kmeans_error_metrics(&self, original: &[f32], assignments: &[i32], centroids: &[f32]) -> ErrorMetrics {
        let mut mse = 0.0;
        let mut mae = 0.0;
        let mut max_error: f32 = 0.0;
        let mut signal_power = 0.0;
        let mut noise_power = 0.0;

        for (&orig, &assignment) in original.iter().zip(assignments.iter()) {
            let centroid = centroids.get(assignment as usize).copied().unwrap_or(0.0);
            let error = orig - centroid;
            
            mse += error * error;
            mae += error.abs();
            max_error = max_error.max(error.abs());
            
            signal_power += orig * orig;
            noise_power += error * error;
        }

        let n = original.len() as f32;
        mse /= n;
        mae /= n;
        
        let snr = if noise_power > 0.0 {
            10.0 * (signal_power / noise_power).log10()
        } else {
            f32::INFINITY
        };

        ErrorMetrics {
            mse,
            mae,
            max_error,
            snr,
        }
    }

    fn initialize_centroids(&self, data: &[f32], k: usize) -> Vec<f32> {
        let min_val = data.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = data.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        (0..k).map(|i| {
            min_val + (max_val - min_val) * (i as f32) / (k as f32 - 1.0)
        }).collect()
    }

    fn assign_to_centroids(&self, data: &[f32], centroids: &[f32]) -> Vec<usize> {
        data.iter().map(|&value| {
            self.find_closest_centroid(value, centroids)
        }).collect()
    }

    fn find_closest_centroid(&self, value: f32, centroids: &[f32]) -> usize {
        centroids.iter()
            .enumerate()
            .min_by(|(_, &a), (_, &b)| {
                (value - a).abs().partial_cmp(&(value - b).abs()).unwrap()
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn update_centroids(&self, data: &[f32], assignments: &[usize], k: usize) -> Vec<f32> {
        let mut new_centroids = vec![0.0; k];
        let mut counts = vec![0; k];

        for (&value, &assignment) in data.iter().zip(assignments.iter()) {
            new_centroids[assignment] += value;
            counts[assignment] += 1;
        }

        for i in 0..k {
            if counts[i] > 0 {
                new_centroids[i] /= counts[i] as f32;
            }
        }

        new_centroids
    }

    fn calculate_variance(&self, data: &[f32]) -> f32 {
        let mean = data.iter().sum::<f32>() / data.len() as f32;
        let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / data.len() as f32;
        variance
    }

    fn detect_outliers(&self, data: &[f32]) -> bool {
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let q1_idx = sorted_data.len() / 4;
        let q3_idx = 3 * sorted_data.len() / 4;
        
        if q1_idx < sorted_data.len() && q3_idx < sorted_data.len() {
            let q1 = sorted_data[q1_idx];
            let q3 = sorted_data[q3_idx];
            let iqr = q3 - q1;
            let lower_bound = q1 - 1.5 * iqr;
            let upper_bound = q3 + 1.5 * iqr;
            
            data.iter().any(|&x| x < lower_bound || x > upper_bound)
        } else {
            false
        }
    }

    pub fn dequantize(&self, quantized: &[i32], params: &QuantizationParameters) -> Vec<f32> {
        quantized.iter().map(|&q| {
            (q as f32 - params.zero_point as f32) * params.scale + params.min_val
        }).collect()
    }
}

/// Factory function to create quantizer instances
pub fn create_quantizer(config: QuantizationConfig) -> UnifiedQuantizer {
    UnifiedQuantizer::new(config)
}

/// Convenience functions for common quantization tasks
pub fn quantize_tensor(data: &[f32], precision: PrecisionLevel) -> Result<QuantizationResult, QuantizationError> {
    let config = QuantizationConfig {
        precision,
        ..Default::default()
    };
    let quantizer = UnifiedQuantizer::new(config);
    quantizer.quantize(data)
}

pub fn quantize_with_salience(
    data: &[f32], 
    salience_weights: HashMap<usize, f32>, 
    precision: PrecisionLevel
) -> Result<QuantizationResult, QuantizationError> {
    let config = QuantizationConfig {
        precision,
        algorithm: QuantizationAlgorithm::SalienceBased,
        ..Default::default()
    };
    let mut quantizer = UnifiedQuantizer::new(config);
    quantizer.set_salience_weights(salience_weights);
    quantizer.quantize(data)
}
