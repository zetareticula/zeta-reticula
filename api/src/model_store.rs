use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use log;
use zeta_vault::{ZetaVault, VaultConfig, KVCache};

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelMetadata {
    path: String,
    name: String,
    compressed: bool,
    bit_depth: Option<String>,
    brand: String,
}

#[derive(Serialize)]
pub struct AvailableModel {
    name: String,
    compressed: bool,
    bit_depth: Option<String>,
}

#[derive(Clone)]
pub struct ModelStore {
    vault: Arc<ZetaVault>,
}

impl ModelStore {
    pub async fn new() -> Self {
        let config = VaultConfig {
            hbm_size: 1000,
            host_size: 10000,
            disk_path: "zeta_vault_db".to_string(),
            pre_load_batch: 10,
        };
        let vault = ZetaVault::new(config).await.unwrap();
        ModelStore { vault: Arc::new(vault) }
    }

    pub async fn add_model(&self, path: String, name: String, brand: String) {
        let metadata = ModelMetadata {
            path,
            name: name.clone(),
            compressed: false,
            bit_depth: None,
            brand,
        };
        let encoded = bincode::serialize(&metadata).unwrap();
        self.vault.store(format!("model_{}", name), encoded).await.unwrap();
        log::info!("Added model: {}", name);
    }

    pub fn get_available_models(&self) -> Vec<AvailableModel> {
        let mut models = Vec::new();
        for entry in self.vault.hbm_cache.iter() {
            if entry.key().starts_with("model_") {
                if let Ok(metadata) = bincode::deserialize::<ModelMetadata>(&entry.value().value) {
                    models.push(AvailableModel {
                        name: metadata.name,
                        compressed: metadata.compressed,
                        bit_depth: metadata.bit_depth,
                    });
                }
            }
        }
        models
    }

    pub fn get_model_path(&self, name: &str) -> Option<String> {
        if let Ok(Some(data)) = self.vault.disk_cache.get(format!("model_{}", name)) {
            if let Ok(metadata) = bincode::deserialize::<ModelMetadata>(&data) {
                return Some(metadata.path);
            }
        }
        None
    }

    #[cfg(feature = "wasm")]
    #[wasm_bindgen]
    pub fn get_available_models_wasm() -> js_sys::Array {
        let model_store = ModelStore::new().await;
        let models = model_store.get_available_models();
        let js_array = js_sys::Array::new_with_length(models.len() as u32);
        for (i, model) in models.iter().enumerate() {
            js_array.set(i as u32, JsValue::from_serde(&model).unwrap());
        }
        js_array
    }
}