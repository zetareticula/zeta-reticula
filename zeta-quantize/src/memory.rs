use crate::cli::PrecisionLevel;
use crate::error::{QuantizationError, Result};
use ndarray::{Array1, Array2};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Memory usage tracker with algebraic assertions
#[derive(Debug, Clone)]
pub struct MemoryTracker {
    /// Layer-wise memory usage in bytes
    layer_memory: HashMap<String, LayerMemory>,
    /// Total original memory
    total_original: u64,
    /// Total quantized memory
    total_quantized: u64,
    /// Safety factor for memory calculations
    safety_factor: f64,
}

#[derive(Debug, Clone)]
pub struct LayerMemory {
    /// Original memory usage in bytes
    pub original: u64,
    /// Quantized memory usage in bytes
    pub quantized: u64,
    /// Number of parameters
    pub param_count: u64,
    /// Original precision
    pub original_precision: PrecisionLevel,
    /// Target precision
    pub target_precision: PrecisionLevel,
}

#[derive(Debug, Clone)]
pub struct MemoryAnalysis {
    /// Memory reduction factor (original / quantized)
    pub reduction_factor: f64,
    /// Total memory saved in bytes
    pub memory_saved: u64,
    /// Per-layer analysis
    pub layer_analysis: HashMap<String, LayerAnalysis>,
    /// Memory vector representation
    pub memory_vector: Array1<f64>,
    /// Compression matrix (layers x precision_levels)
    pub compression_matrix: Array2<f64>,
}

#[derive(Debug, Clone)]
pub struct LayerAnalysis {
    /// Layer reduction factor
    pub reduction_factor: f64,
    /// Memory saved in this layer
    pub memory_saved: u64,
    /// Theoretical vs actual compression ratio
    pub compression_efficiency: f64,
}

impl MemoryTracker {
    pub fn new(safety_factor: f64) -> Self {
        Self {
            layer_memory: HashMap::new(),
            total_original: 0,
            total_quantized: 0,
            safety_factor,
        }
    }

    /// Add memory usage for a layer
    pub fn add_layer(
        &mut self,
        layer_name: String,
        param_count: u64,
        original_precision: PrecisionLevel,
        target_precision: PrecisionLevel,
    ) {
        let original_mem = param_count * (original_precision.bytes_per_element() as u64);
        let quantized_mem = param_count * (target_precision.bytes_per_element().ceil() as u64);

        let layer_mem = LayerMemory {
            original: original_mem,
            quantized: quantized_mem,
            param_count,
            original_precision,
            target_precision,
        };

        self.total_original += original_mem;
        self.total_quantized += quantized_mem;
        self.layer_memory.insert(layer_name, layer_mem);

        debug!("Added layer memory: {} -> {} bytes", original_mem, quantized_mem);
    }

    /// Perform algebraic memory assertions
    pub fn validate_memory_constraints(&self, max_memory_gb: Option<f64>) -> Result<()> {
        // Assertion 1: Quantized memory must be less than original
        if self.total_quantized >= self.total_original {
            return Err(QuantizationError::memory_assertion(
                self.total_original,
                self.total_quantized,
            ));
        }

        // Assertion 2: Memory reduction factor must be > 1
        let reduction_factor = self.total_original as f64 / self.total_quantized as f64;
        if reduction_factor <= 1.0 {
            return Err(QuantizationError::Memory(format!(
                "Invalid reduction factor: {:.2}",
                reduction_factor
            )));
        }

        // Assertion 3: Check against memory limit with safety factor
        if let Some(max_gb) = max_memory_gb {
            let max_bytes = (max_gb * 1024.0 * 1024.0 * 1024.0) as u64;
            let safe_limit = (max_bytes as f64 / self.safety_factor) as u64;
            
            if self.total_quantized > safe_limit {
                return Err(QuantizationError::Memory(format!(
                    "Memory usage {} exceeds safe limit {} ({}GB with safety factor {:.1})",
                    format_bytes(self.total_quantized),
                    format_bytes(safe_limit),
                    max_gb,
                    self.safety_factor
                )));
            }
        }

        // Assertion 4: Per-layer validation
        for (layer_name, layer_mem) in &self.layer_memory {
            let theoretical_reduction = layer_mem.original_precision.memory_reduction_factor();
            let actual_reduction = layer_mem.original as f64 / layer_mem.quantized as f64;
            
            // Allow some tolerance for rounding and overhead
            if actual_reduction < theoretical_reduction * 0.8 {
                warn!(
                    "Layer {} has lower than expected compression: {:.2}x vs {:.2}x theoretical",
                    layer_name, actual_reduction, theoretical_reduction
                );
            }
        }

        info!("Memory validation passed: {:.2}x reduction", reduction_factor);
        Ok(())
    }

    /// Generate comprehensive memory analysis
    pub fn analyze(&self) -> MemoryAnalysis {
        let reduction_factor = if self.total_quantized > 0 {
            self.total_original as f64 / self.total_quantized as f64
        } else {
            0.0
        };

        let memory_saved = self.total_original.saturating_sub(self.total_quantized);

        // Create memory vector (memory usage per layer)
        let layer_count = self.layer_memory.len();
        let mut memory_vector = Array1::zeros(layer_count);
        let mut layer_analysis = HashMap::new();

        for (i, (layer_name, layer_mem)) in self.layer_memory.iter().enumerate() {
            memory_vector[i] = layer_mem.quantized as f64;

            let layer_reduction = if layer_mem.quantized > 0 {
                layer_mem.original as f64 / layer_mem.quantized as f64
            } else {
                0.0
            };

            let theoretical_reduction = layer_mem.original_precision.memory_reduction_factor();
            let compression_efficiency = layer_reduction / theoretical_reduction;

            layer_analysis.insert(
                layer_name.clone(),
                LayerAnalysis {
                    reduction_factor: layer_reduction,
                    memory_saved: layer_mem.original.saturating_sub(layer_mem.quantized),
                    compression_efficiency,
                },
            );
        }

        // Create compression matrix (layers x precision levels)
        let precision_levels = [
            PrecisionLevel::Fp32,
            PrecisionLevel::Fp16,
            PrecisionLevel::Int8,
            PrecisionLevel::Int4,
            PrecisionLevel::Int2,
            PrecisionLevel::Int1,
        ];
        
        let mut compression_matrix = Array2::zeros((layer_count, precision_levels.len()));
        
        for (i, (_, layer_mem)) in self.layer_memory.iter().enumerate() {
            for (j, &precision) in precision_levels.iter().enumerate() {
                let theoretical_mem = layer_mem.param_count * (precision.bytes_per_element().ceil() as u64);
                compression_matrix[[i, j]] = layer_mem.original as f64 / theoretical_mem as f64;
            }
        }

        MemoryAnalysis {
            reduction_factor,
            memory_saved,
            layer_analysis,
            memory_vector,
            compression_matrix,
        }
    }

    /// Calculate peak memory usage during quantization
    pub fn estimate_peak_memory(&self) -> u64 {
        // Peak memory = original + quantized + working memory
        // Working memory is estimated as 20% of original for intermediate calculations
        let working_memory = (self.total_original as f64 * 0.2) as u64;
        self.total_original + self.total_quantized + working_memory
    }

    /// Compound memory assertions across multiple models
    pub fn compound_assertion(trackers: &[&MemoryTracker]) -> Result<CompoundAnalysis> {
        let total_original: u64 = trackers.iter().map(|t| t.total_original).sum();
        let total_quantized: u64 = trackers.iter().map(|t| t.total_quantized).sum();
        
        if total_quantized >= total_original {
            return Err(QuantizationError::memory_assertion(total_original, total_quantized));
        }

        let compound_reduction = total_original as f64 / total_quantized as f64;
        let total_saved = total_original - total_quantized;

        // Create compound memory vector
        let total_layers: usize = trackers.iter().map(|t| t.layer_memory.len()).sum();
        let mut compound_vector = Array1::zeros(total_layers);
        
        let mut idx = 0;
        for tracker in trackers {
            for layer_mem in tracker.layer_memory.values() {
                compound_vector[idx] = layer_mem.quantized as f64;
                idx += 1;
            }
        }

        // Algebraic validation: sum of individual reductions should approximate compound
        let individual_sum: f64 = trackers
            .iter()
            .map(|t| t.total_original as f64 / t.total_quantized.max(1) as f64)
            .sum();
        
        let expected_compound = individual_sum / trackers.len() as f64;
        let variance = (compound_reduction - expected_compound).abs() / expected_compound;
        
        if variance > 0.1 {
            warn!("Compound reduction variance: {:.2}%", variance * 100.0);
        }

        Ok(CompoundAnalysis {
            total_reduction_factor: compound_reduction,
            total_memory_saved: total_saved,
            model_count: trackers.len(),
            compound_vector,
            variance,
        })
    }

    pub fn total_original(&self) -> u64 {
        self.total_original
    }

    pub fn total_quantized(&self) -> u64 {
        self.total_quantized
    }
}

#[derive(Debug)]
pub struct CompoundAnalysis {
    pub total_reduction_factor: f64,
    pub total_memory_saved: u64,
    pub model_count: usize,
    pub compound_vector: Array1<f64>,
    pub variance: f64,
}

/// Format bytes in human-readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new(1.2);
        
        tracker.add_layer(
            "layer1".to_string(),
            1000,
            PrecisionLevel::Fp32,
            PrecisionLevel::Int8,
        );

        assert_eq!(tracker.total_original(), 4000); // 1000 * 4 bytes
        assert_eq!(tracker.total_quantized(), 1000); // 1000 * 1 byte
        
        let analysis = tracker.analyze();
        assert_eq!(analysis.reduction_factor, 4.0);
    }

    #[test]
    fn test_memory_validation() {
        let mut tracker = MemoryTracker::new(1.2);
        
        tracker.add_layer(
            "layer1".to_string(),
            1000,
            PrecisionLevel::Fp32,
            PrecisionLevel::Int8,
        );

        assert!(tracker.validate_memory_constraints(Some(1.0)).is_ok());
        assert!(tracker.validate_memory_constraints(Some(0.001)).is_err());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
}
