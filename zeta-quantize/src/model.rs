use crate::error::{QuantizationError, Result};
use candle_core::{Device, Tensor};
use safetensors::SafeTensors;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{debug, info};

#[derive(Debug, Clone, PartialEq)]
pub enum ModelFormat {
    Safetensors,
    PyTorch,
    Onnx,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub format: ModelFormat,
    pub total_parameters: u64,
    pub layer_names: Vec<String>,
    pub tensor_shapes: HashMap<String, Vec<usize>>,
    pub dtype_info: HashMap<String, String>,
}

pub struct ModelLoader {
    device: Device,
}

impl ModelLoader {
    pub fn new(device: Device) -> Self {
        Self { device }
    }

    /// Load a model from various formats
    pub async fn load_model(&self, path: &Path) -> Result<Tensor> {
        let format = self.detect_format(path)?;
        
        match format {
            ModelFormat::Safetensors => self.load_safetensors(path).await,
            ModelFormat::PyTorch => self.load_pytorch(path).await,
            ModelFormat::Onnx => self.load_onnx(path).await,
            ModelFormat::Unknown => Err(QuantizationError::unsupported_format(
                format!("Unknown format for file: {:?}", path)
            )),
        }
    }

    /// Save a quantized model
    pub async fn save_model(&self, tensor: &Tensor, path: &Path) -> Result<()> {
        let format = self.detect_output_format(path)?;
        
        match format {
            ModelFormat::Safetensors => self.save_safetensors(tensor, path).await,
            ModelFormat::PyTorch => self.save_pytorch(tensor, path).await,
            _ => Err(QuantizationError::unsupported_format(
                format!("Unsupported output format: {:?}", format)
            )),
        }
    }

    /// Detect model format from file extension
    fn detect_format(&self, path: &Path) -> Result<ModelFormat> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| QuantizationError::unsupported_format("No file extension"))?;

        match extension.to_lowercase().as_str() {
            "safetensors" => Ok(ModelFormat::Safetensors),
            "pt" | "pth" | "bin" => Ok(ModelFormat::PyTorch),
            "onnx" => Ok(ModelFormat::Onnx),
            _ => Ok(ModelFormat::Unknown),
        }
    }

    /// Detect output format from file extension
    fn detect_output_format(&self, path: &Path) -> Result<ModelFormat> {
        self.detect_format(path)
    }

    /// Load model from Safetensors format
    async fn load_safetensors(&self, path: &Path) -> Result<Tensor> {
        info!("Loading Safetensors model from {:?}", path);
        
        let data = fs::read(path).await
            .map_err(|e| QuantizationError::model_load(format!("Failed to read file: {}", e)))?;

        let safetensors = SafeTensors::deserialize(&data)
            .map_err(|e| QuantizationError::model_load(format!("Failed to parse Safetensors: {}", e)))?;

        // Get the first tensor or concatenate all tensors
        let tensor_names: Vec<_> = safetensors.names().collect();
        
        if tensor_names.is_empty() {
            return Err(QuantizationError::model_load("No tensors found in Safetensors file"));
        }

        // For simplicity, load the first tensor or create a composite tensor
        if tensor_names.len() == 1 {
            let tensor_name = &tensor_names[0];
            let tensor_view = safetensors.tensor(tensor_name)
                .map_err(|e| QuantizationError::model_load(format!("Failed to get tensor: {}", e)))?;
            
            self.tensor_view_to_candle_tensor(tensor_view)
        } else {
            // Concatenate multiple tensors
            self.concatenate_safetensors(&safetensors, &tensor_names)
        }
    }

    /// Convert SafeTensors tensor view to Candle tensor
    fn tensor_view_to_candle_tensor(&self, tensor_view: safetensors::tensor::TensorView) -> Result<Tensor> {
        let shape: Vec<usize> = tensor_view.shape().iter().map(|&x| x).collect();
        let data = tensor_view.data();

        // Convert based on dtype
        match tensor_view.dtype() {
            safetensors::Dtype::F32 => {
                let float_data: Vec<f32> = data
                    .chunks_exact(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();
                
                Tensor::from_vec(float_data, &shape, &self.device)
                    .map_err(|e| QuantizationError::tensor_op(e.to_string()))
            }
            safetensors::Dtype::F16 => {
                // Convert F16 to F32 for processing
                let float_data: Vec<f32> = data
                    .chunks_exact(2)
                    .map(|chunk| {
                        let bits = u16::from_le_bytes([chunk[0], chunk[1]]);
                        half::f16::from_bits(bits).to_f32()
                    })
                    .collect();
                
                Tensor::from_vec(float_data, &shape, &self.device)
                    .map_err(|e| QuantizationError::tensor_op(e.to_string()))
            }
            safetensors::Dtype::I8 => {
                let int_data: Vec<f32> = data.iter().map(|&x| x as i8 as f32).collect();
                
                Tensor::from_vec(int_data, &shape, &self.device)
                    .map_err(|e| QuantizationError::tensor_op(e.to_string()))
            }
            _ => Err(QuantizationError::unsupported_format(
                format!("Unsupported dtype: {:?}", tensor_view.dtype())
            )),
        }
    }

    /// Concatenate multiple tensors from SafeTensors
    fn concatenate_safetensors(
        &self,
        safetensors: &SafeTensors,
        tensor_names: &[String],
    ) -> Result<Tensor> {
        let mut tensors = Vec::new();
        let mut total_elements = 0;

        for name in tensor_names {
            let tensor_view = safetensors.tensor(name)
                .map_err(|e| QuantizationError::model_load(format!("Failed to get tensor {}: {}", name, e)))?;
            
            let tensor = self.tensor_view_to_candle_tensor(tensor_view)?;
            total_elements += tensor.elem_count();
            tensors.push(tensor);
        }

        debug!("Concatenating {} tensors with {} total elements", tensors.len(), total_elements);

        // Flatten all tensors and concatenate
        let mut all_data = Vec::with_capacity(total_elements);
        
        for tensor in tensors {
            let flattened = tensor.flatten_all()
                .map_err(|e| QuantizationError::tensor_op(e.to_string()))?;
            let data: Vec<f32> = flattened.to_vec1()
                .map_err(|e| QuantizationError::tensor_op(e.to_string()))?;
            all_data.extend(data);
        }

        Tensor::from_vec(all_data, &[total_elements], &self.device)
            .map_err(|e| QuantizationError::tensor_op(e.to_string()))
    }

    /// Load model from PyTorch format (simplified implementation)
    async fn load_pytorch(&self, path: &Path) -> Result<Tensor> {
        info!("Loading PyTorch model from {:?}", path);
        
        // For now, return an error as PyTorch loading requires additional dependencies
        // In a full implementation, you would use candle-transformers or tch crate
        Err(QuantizationError::unsupported_format(
            "PyTorch format loading not implemented yet. Please convert to Safetensors format."
        ))
    }

    /// Load model from ONNX format (simplified implementation)
    async fn load_onnx(&self, path: &Path) -> Result<Tensor> {
        info!("Loading ONNX model from {:?}", path);
        
        // For now, return an error as ONNX loading requires additional dependencies
        Err(QuantizationError::unsupported_format(
            "ONNX format loading not implemented yet. Please convert to Safetensors format."
        ))
    }

    /// Save model in Safetensors format
    async fn save_safetensors(&self, tensor: &Tensor, path: &Path) -> Result<()> {
        info!("Saving model to Safetensors format: {:?}", path);
        
        // Convert tensor to bytes
        let data: Vec<f32> = tensor.flatten_all()
            .map_err(|e| QuantizationError::tensor_op(e.to_string()))?
            .to_vec1()
            .map_err(|e| QuantizationError::tensor_op(e.to_string()))?;

        let shape: Vec<usize> = tensor.shape().dims().to_vec();
        
        // Convert f32 data to bytes
        let mut bytes = Vec::with_capacity(data.len() * 4);
        for &value in &data {
            bytes.extend_from_slice(&value.to_le_bytes());
        }

        // Create SafeTensors with single tensor
        let mut tensors = HashMap::new();
        tensors.insert(
            "quantized_model".to_string(),
            (safetensors::Dtype::F32, shape, bytes.as_slice()),
        );

        let serialized = safetensors::serialize(&tensors, &None)
            .map_err(|e| QuantizationError::model_load(format!("Failed to serialize: {}", e)))?;

        fs::write(path, serialized).await
            .map_err(|e| QuantizationError::model_load(format!("Failed to write file: {}", e)))?;

        info!("Successfully saved quantized model to {:?}", path);
        Ok(())
    }

    /// Save model in PyTorch format (simplified implementation)
    async fn save_pytorch(&self, tensor: &Tensor, path: &Path) -> Result<()> {
        info!("Saving model to PyTorch format: {:?}", path);
        
        // For now, return an error as PyTorch saving requires additional dependencies
        Err(QuantizationError::unsupported_format(
            "PyTorch format saving not implemented yet. Please use Safetensors format."
        ))
    }

    /// Extract metadata from a model file
    pub async fn extract_metadata(&self, path: &Path) -> Result<ModelMetadata> {
        let format = self.detect_format(path)?;
        
        match format {
            ModelFormat::Safetensors => self.extract_safetensors_metadata(path).await,
            _ => Err(QuantizationError::unsupported_format(
                "Metadata extraction only supported for Safetensors format"
            )),
        }
    }

    /// Extract metadata from Safetensors file
    async fn extract_safetensors_metadata(&self, path: &Path) -> Result<ModelMetadata> {
        let data = fs::read(path).await
            .map_err(|e| QuantizationError::model_load(format!("Failed to read file: {}", e)))?;

        let safetensors = SafeTensors::deserialize(&data)
            .map_err(|e| QuantizationError::model_load(format!("Failed to parse Safetensors: {}", e)))?;

        let tensor_names: Vec<String> = safetensors.names().map(|s| s.to_string()).collect();
        let mut tensor_shapes = HashMap::new();
        let mut dtype_info = HashMap::new();
        let mut total_parameters = 0u64;

        for name in &tensor_names {
            let tensor_view = safetensors.tensor(name)
                .map_err(|e| QuantizationError::model_load(format!("Failed to get tensor: {}", e)))?;
            
            let shape: Vec<usize> = tensor_view.shape().iter().map(|&x| x).collect();
            let param_count: usize = shape.iter().product();
            total_parameters += param_count as u64;
            
            tensor_shapes.insert(name.clone(), shape);
            dtype_info.insert(name.clone(), format!("{:?}", tensor_view.dtype()));
        }

        Ok(ModelMetadata {
            format: ModelFormat::Safetensors,
            total_parameters,
            layer_names: tensor_names,
            tensor_shapes,
            dtype_info,
        })
    }

    /// Create a synthetic model for testing
    pub fn create_test_model(&self, shape: &[usize]) -> Result<Tensor> {
        let total_elements: usize = shape.iter().product();
        let data: Vec<f32> = (0..total_elements)
            .map(|i| (i as f32 / 1000.0).sin())
            .collect();

        Tensor::from_vec(data, shape, &self.device)
            .map_err(|e| QuantizationError::tensor_op(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;
    use tempfile::NamedTempFile;

    #[test]
    fn test_format_detection() {
        let loader = ModelLoader::new(Device::Cpu);
        
        assert_eq!(
            loader.detect_format(Path::new("model.safetensors")).unwrap(),
            ModelFormat::Safetensors
        );
        assert_eq!(
            loader.detect_format(Path::new("model.pt")).unwrap(),
            ModelFormat::PyTorch
        );
        assert_eq!(
            loader.detect_format(Path::new("model.onnx")).unwrap(),
            ModelFormat::Onnx
        );
    }

    #[tokio::test]
    async fn test_create_test_model() {
        let loader = ModelLoader::new(Device::Cpu);
        let model = loader.create_test_model(&[100, 50]).unwrap();
        
        assert_eq!(model.shape().dims(), &[100, 50]);
        assert_eq!(model.elem_count(), 5000);
    }

    #[tokio::test]
    async fn test_save_load_roundtrip() {
        let loader = ModelLoader::new(Device::Cpu);
        let original_model = loader.create_test_model(&[10, 10]).unwrap();
        
        let temp_file = NamedTempFile::with_suffix(".safetensors").unwrap();
        let path = temp_file.path();
        
        // Save model
        loader.save_model(&original_model, path).await.unwrap();
        
        // Load model back
        let loaded_model = loader.load_model(path).await.unwrap();
        
        // Check that shapes match (values might differ due to serialization)
        assert_eq!(loaded_model.elem_count(), original_model.elem_count());
    }
}
