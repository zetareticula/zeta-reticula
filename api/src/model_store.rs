use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use log;

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelMetadata {
    path: String,
    name: String,
    compressed: bool,
    bit_depth: Option<String>,
    brand: String, // Reflecting the Zeta Reticula logo
}

#[derive(Serialize)]
pub struct AvailableModel {
    name: String,
    compressed: bool,
    bit_depth: Option<String>,
}

#[derive(Clone)]
pub struct ModelStore {
    models: Arc<RwLock<HashMap<String, ModelMetadata>>>,
}

impl ModelStore {
    pub async fn new() -> Self {
        let mut models = HashMap::new();
        // Preload compressed models with Zeta Reticula branding
        models.insert("qwen".to_string(), ModelMetadata {
            path: "preloaded_qwen.bin".to_string(),
            name: "Qwen".to_string(),
            compressed: true,
            bit_depth: Some("8".to_string()),
            brand: "Zeta Reticula".to_string(),
        });
        models.insert("distilbert".to_string(), ModelMetadata {
            path: "preloaded_distilbert.bin".to_string(),
            name: "DistilBERT".to_string(),
            compressed: true,
            bit_depth: Some("8".to_string()),
            brand: "Zeta Reticula".to_string(),
        });
        models.insert("deepseek".to_string(), ModelMetadata {
            path: "preloaded_deepseek.bin".to_string(),
            name: "DeepSeek".to_string(),
            compressed: true,
            bit_depth: Some("8".to_string()),
            brand: "Zeta Reticula".to_string(),
        });
        models.insert("huggingface_bert".to_string(), ModelMetadata {
            path: "preloaded_bert.bin".to_string(),
            name: "HuggingFace BERT".to_string(),
            compressed: true,
            bit_depth: Some("8".to_string()),
            brand: "Zeta Reticula".to_string(),
        });

        ModelStore {
            models: Arc::new(RwLock::new(models)),
        }
    }

    pub async fn add_model(&self, path: String, name: String, brand: String) {
        let mut models = self.models.write().await;
        models.insert(name.clone(), ModelMetadata {
            path,
            name,
            compressed: false,
            bit_depth: None,
            brand,
        });
        log::info!("Added model: {}", name);
    }

    pub fn get_available_models(&self) -> Vec<AvailableModel> {
        let models = self.models.read().unwrap();
        models.values().map(|m| AvailableModel {
            name: m.name.clone(),
            compressed: m.compressed,
            bit_depth: m.bit_depth.clone(),
        }).collect()
    }

    pub fn get_model_path(&self, name: &str) -> Option<String> {
        let models = self.models.read().unwrap();
        models.get(name).map(|m| m.path.clone())
    }
}