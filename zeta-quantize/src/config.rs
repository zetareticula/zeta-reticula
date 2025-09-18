use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::error::{QuantizationError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Memory management settings
    pub memory: MemoryConfig,
    
    /// Quantization algorithm settings
    pub quantization: QuantizationConfig,
    
    /// Performance optimization settings
    pub performance: PerformanceConfig,
    
    /// Validation settings
    pub validation: ValidationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum memory usage in GB
    pub max_memory_gb: Option<f64>,
    
    /// Enable memory mapping for large models
    pub use_memory_mapping: bool,
    
    /// Chunk size for processing in MB
    pub chunk_size_mb: usize,
    
    /// Enable algebraic memory assertions
    pub enable_memory_assertions: bool,
    
    /// Memory safety factor (multiplier for safety margins)
    pub safety_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationConfig {
    /// Default calibration dataset size
    pub calibration_samples: usize,
    
    /// Enable symmetric quantization
    pub symmetric: bool,
    
    /// Per-channel quantization
    pub per_channel: bool,
    
    /// Quantization algorithm
    pub algorithm: QuantizationAlgorithm,
    
    /// Outlier threshold for handling extreme values
    pub outlier_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantizationAlgorithm {
    /// Linear quantization with uniform scaling
    Linear,
    /// K-means clustering based quantization
    KMeans,
    /// Learned quantization with gradient descent
    Learned,
    /// Block-wise quantization
    BlockWise { block_size: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of threads for parallel processing
    pub num_threads: Option<usize>,
    
    /// Enable GPU acceleration if available
    pub use_gpu: bool,
    
    /// Batch size for tensor operations
    pub batch_size: usize,
    
    /// Enable fast math optimizations
    pub fast_math: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Enable model validation after quantization
    pub validate_output: bool,
    
    /// Maximum acceptable accuracy loss (percentage)
    pub max_accuracy_loss: f64,
    
    /// Number of validation samples
    pub validation_samples: usize,
    
    /// Enable statistical validation of quantized weights
    pub statistical_validation: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            memory: MemoryConfig {
                max_memory_gb: None,
                use_memory_mapping: true,
                chunk_size_mb: 512,
                enable_memory_assertions: true,
                safety_factor: 1.2,
            },
            quantization: QuantizationConfig {
                calibration_samples: 1000,
                symmetric: true,
                per_channel: true,
                algorithm: QuantizationAlgorithm::Linear,
                outlier_threshold: 3.0,
            },
            performance: PerformanceConfig {
                num_threads: None, // Use system default
                use_gpu: false,
                batch_size: 32,
                fast_math: true,
            },
            validation: ValidationConfig {
                validate_output: true,
                max_accuracy_loss: 5.0, // 5% max loss
                validation_samples: 100,
                statistical_validation: true,
            },
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| QuantizationError::config(format!("Failed to read config file: {}", e)))?;
        
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    /// Save configuration to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| QuantizationError::config(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(path, content)
            .map_err(|e| QuantizationError::config(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
    
    /// Validate configuration parameters
    pub fn validate(&self) -> Result<()> {
        if let Some(max_mem) = self.memory.max_memory_gb {
            if max_mem <= 0.0 {
                return Err(QuantizationError::config("Max memory must be positive"));
            }
        }
        
        if self.memory.chunk_size_mb == 0 {
            return Err(QuantizationError::config("Chunk size must be positive"));
        }
        
        if self.memory.safety_factor <= 0.0 {
            return Err(QuantizationError::config("Safety factor must be positive"));
        }
        
        if self.quantization.calibration_samples == 0 {
            return Err(QuantizationError::config("Calibration samples must be positive"));
        }
        
        if self.validation.max_accuracy_loss < 0.0 || self.validation.max_accuracy_loss > 100.0 {
            return Err(QuantizationError::config("Max accuracy loss must be between 0 and 100"));
        }
        
        Ok(())
    }
    
    /// Get effective number of threads
    pub fn effective_threads(&self) -> usize {
        self.performance.num_threads.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(1)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let temp_file = NamedTempFile::new().unwrap();
        
        config.save_to_file(temp_file.path()).unwrap();
        let loaded_config = Config::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.memory.chunk_size_mb, loaded_config.memory.chunk_size_mb);
        assert_eq!(config.quantization.symmetric, loaded_config.quantization.symmetric);
    }
    
    #[test]
    fn test_invalid_config() {
        let mut config = Config::default();
        config.memory.chunk_size_mb = 0;
        
        assert!(config.validate().is_err());
    }
}
