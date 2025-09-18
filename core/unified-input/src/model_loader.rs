use crate::error::{UnifiedInputError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use candle_core::Tensor;

/// Supported model formats
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModelFormat {
    SafeTensors,
    PyTorch,
    HuggingFaceJson,
    GGUF,
    ONNX,
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub cache_dir: Option<PathBuf>,
    pub supported_formats: Vec<ModelFormat>,
    pub auto_detect_format: bool,
    pub download_timeout: u64,
    pub max_file_size: u64,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            cache_dir: None,
            supported_formats: vec![
                ModelFormat::SafeTensors,
                ModelFormat::HuggingFaceJson,
                ModelFormat::PyTorch,
            ],
            auto_detect_format: true,
            download_timeout: 300, // 5 minutes
            max_file_size: 10 * 1024 * 1024 * 1024, // 10GB
        }
    }
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub format: ModelFormat,
    pub path: PathBuf,
    pub size: u64,
    pub vocab_size: Option<usize>,
    pub embedding_dim: Option<usize>,
    pub num_layers: Option<usize>,
    pub config: HashMap<String, serde_json::Value>,
}

/// Unified model loader that handles multiple formats
#[derive(Debug)]
pub struct ModelLoader {
    config: ModelConfig,
    loaded_models: HashMap<String, ModelMetadata>,
    hf_api: Option<hf_hub::api::tokio::Api>,
}

impl ModelLoader {
    /// Create a new model loader
    pub async fn new(config: ModelConfig) -> Result<Self> {
        let hf_api = Some(
            if let Some(cache_dir) = &config.cache_dir {
                hf_hub::api::tokio::Api::new()?.with_cache_dir(cache_dir.clone())
            } else {
                hf_hub::api::tokio::Api::new()?
            }
        );

        Ok(Self {
            config,
            loaded_models: HashMap::new(),
            hf_api,
        })
    }

    /// Load a model from various sources
    pub async fn load_model(&mut self, model_path: &str) -> Result<ModelMetadata> {
        // Check if already loaded
        if let Some(metadata) = self.loaded_models.get(model_path) {
            return Ok(metadata.clone());
        }

        let metadata = if model_path.starts_with("http") || !Path::new(model_path).exists() {
            // Try to load from HuggingFace Hub
            self.load_from_hub(model_path).await?
        } else {
            // Load from local path
            self.load_from_local(model_path).await?
        };

        self.loaded_models.insert(model_path.to_string(), metadata.clone());
        Ok(metadata)
    }

    /// Load model from HuggingFace Hub
    async fn load_from_hub(&self, model_name: &str) -> Result<ModelMetadata> {
        let api = self.hf_api.as_ref()
            .ok_or_else(|| UnifiedInputError::Config("HuggingFace API not initialized".to_string()))?;

        let repo = api.model(model_name.to_string());

        // Try to find model files in order of preference
        let mut model_path = None;
        let mut format = ModelFormat::SafeTensors;

        // Check for safetensors first
        if let Ok(path) = repo.get("model.safetensors").await {
            model_path = Some(path);
            format = ModelFormat::SafeTensors;
        } else if let Ok(path) = repo.get("pytorch_model.bin").await {
            model_path = Some(path);
            format = ModelFormat::PyTorch;
        } else if let Ok(path) = repo.get("config.json").await {
            model_path = Some(path);
            format = ModelFormat::HuggingFaceJson;
        }

        let path = model_path.ok_or_else(|| {
            UnifiedInputError::ModelNotFound(format!("No supported model files found for {}", model_name))
        })?;

        // Load config.json for metadata
        let config = self.load_model_config(&repo).await.unwrap_or_default();
        
        let metadata = std::fs::metadata(&path)?;
        
        Ok(ModelMetadata {
            name: model_name.to_string(),
            format,
            path,
            size: metadata.len(),
            vocab_size: self.extract_vocab_size(&config),
            embedding_dim: self.extract_embedding_dim(&config),
            num_layers: self.extract_num_layers(&config),
            config,
        })
    }

    /// Load model from local path
    async fn load_from_local(&self, model_path: &str) -> Result<ModelMetadata> {
        let path = PathBuf::from(model_path);
        
        if !path.exists() {
            return Err(UnifiedInputError::ModelNotFound(format!("Model file not found: {}", model_path)));
        }

        let format = if self.config.auto_detect_format {
            self.detect_format(&path)?
        } else {
            ModelFormat::SafeTensors // Default
        };

        let metadata = std::fs::metadata(&path)?;
        let config = self.load_local_config(&path).await.unwrap_or_default();

        Ok(ModelMetadata {
            name: path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            format,
            path,
            size: metadata.len(),
            vocab_size: self.extract_vocab_size(&config),
            embedding_dim: self.extract_embedding_dim(&config),
            num_layers: self.extract_num_layers(&config),
            config,
        })
    }

    /// Detect model format from file extension
    fn detect_format(&self, path: &Path) -> Result<ModelFormat> {
        match path.extension().and_then(|s| s.to_str()) {
            Some("safetensors") => Ok(ModelFormat::SafeTensors),
            Some("bin") | Some("pt") | Some("pth") => Ok(ModelFormat::PyTorch),
            Some("json") => Ok(ModelFormat::HuggingFaceJson),
            Some("gguf") => Ok(ModelFormat::GGUF),
            Some("onnx") => Ok(ModelFormat::ONNX),
            _ => Err(UnifiedInputError::InvalidFormat(
                format!("Cannot detect format for file: {:?}", path)
            )),
        }
    }

    /// Load model configuration from HuggingFace repo
    async fn load_model_config(&self, repo: &hf_hub::api::tokio::ApiRepo) -> Result<HashMap<String, serde_json::Value>> {
        match repo.get("config.json").await {
            Ok(config_path) => {
                let config_content = tokio::fs::read_to_string(config_path).await?;
                let config: HashMap<String, serde_json::Value> = serde_json::from_str(&config_content)?;
                Ok(config)
            }
            Err(_) => Ok(HashMap::new()),
        }
    }

    /// Load local model configuration
    async fn load_local_config(&self, model_path: &Path) -> Result<HashMap<String, serde_json::Value>> {
        let config_path = model_path.parent()
            .map(|p| p.join("config.json"))
            .unwrap_or_else(|| PathBuf::from("config.json"));

        if config_path.exists() {
            let config_content = tokio::fs::read_to_string(config_path).await?;
            let config: HashMap<String, serde_json::Value> = serde_json::from_str(&config_content)?;
            Ok(config)
        } else {
            Ok(HashMap::new())
        }
    }

    /// Extract vocabulary size from config
    fn extract_vocab_size(&self, config: &HashMap<String, serde_json::Value>) -> Option<usize> {
        config.get("vocab_size")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
    }

    /// Extract embedding dimension from config
    fn extract_embedding_dim(&self, config: &HashMap<String, serde_json::Value>) -> Option<usize> {
        config.get("hidden_size")
            .or_else(|| config.get("d_model"))
            .or_else(|| config.get("embedding_size"))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
    }

    /// Extract number of layers from config
    fn extract_num_layers(&self, config: &HashMap<String, serde_json::Value>) -> Option<usize> {
        config.get("num_hidden_layers")
            .or_else(|| config.get("n_layer"))
            .or_else(|| config.get("num_layers"))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
    }

    /// Load safetensors file
    pub async fn load_safetensors(&self, path: &Path) -> Result<HashMap<String, Tensor>> {
        use safetensors::SafeTensors;
        
        let data = tokio::fs::read(path).await?;
        let safetensors = SafeTensors::deserialize(&data)?;
        
        let mut tensors = HashMap::new();
        
        for (name, tensor_view) in safetensors.tensors() {
            // Convert safetensors data to Candle tensor
            // This is a simplified conversion - in practice you'd need proper dtype handling
            let shape: Vec<usize> = tensor_view.shape().to_vec();
            let data = tensor_view.data();
            
            // Create tensor (simplified - would need proper dtype conversion)
            let tensor = Tensor::from_raw_buffer(
                data,
                candle_core::DType::F32, // Simplified
                &shape,
                &candle_core::Device::Cpu,
            )?;
            
            tensors.insert(name.to_string(), tensor);
        }
        
        Ok(tensors)
    }

    /// Get loaded model metadata
    pub fn get_model_metadata(&self, model_path: &str) -> Option<&ModelMetadata> {
        self.loaded_models.get(model_path)
    }

    /// List all loaded models
    pub fn list_loaded_models(&self) -> Vec<&ModelMetadata> {
        self.loaded_models.values().collect()
    }

    /// Clear loaded models cache
    pub fn clear_cache(&mut self) {
        self.loaded_models.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_format_detection() {
        let config = ModelConfig::default();
        let loader = ModelLoader::new(config).await.unwrap();

        assert_eq!(
            loader.detect_format(Path::new("model.safetensors")).unwrap(),
            ModelFormat::SafeTensors
        );
        assert_eq!(
            loader.detect_format(Path::new("model.bin")).unwrap(),
            ModelFormat::PyTorch
        );
        assert_eq!(
            loader.detect_format(Path::new("config.json")).unwrap(),
            ModelFormat::HuggingFaceJson
        );
    }

    #[tokio::test]
    async fn test_config_extraction() {
        let config = ModelConfig::default();
        let loader = ModelLoader::new(config).await.unwrap();

        let mut model_config = HashMap::new();
        model_config.insert("vocab_size".to_string(), serde_json::Value::Number(serde_json::Number::from(50257)));
        model_config.insert("hidden_size".to_string(), serde_json::Value::Number(serde_json::Number::from(768)));
        model_config.insert("num_hidden_layers".to_string(), serde_json::Value::Number(serde_json::Number::from(12)));

        assert_eq!(loader.extract_vocab_size(&model_config), Some(50257));
        assert_eq!(loader.extract_embedding_dim(&model_config), Some(768));
        assert_eq!(loader.extract_num_layers(&model_config), Some(12));
    }
}
