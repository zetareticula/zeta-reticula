use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use salience_engine::quantizer::{PrecisionLevel, QuantizationResult};
use model_store::ModelStore;
use thiserror::Error;
use validator::Validate;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "wasm")]
use wasm_bindgen_futures::future_to_promise;
use ndarray::{Array2, array};
use half::f16;

#[derive(Error, Debug)]
enum QuantizationError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Validate, Deserialize)]
pub struct QuantizationRequest {
    #[validate(length(min = 1, message = "Model name is required"))]
    model_name: String,
    #[validate(custom = "validate_bit_depth")]
    bit_depth: String,
}

fn validate_bit_depth(bit_depth: &str) -> Result<(), validator::ValidationError> {
    match bit_depth {
        "2" | "4" | "8" | "16" => Ok(()), // Added 2-bit support
        _ => Err(validator::ValidationError::new("Bit depth must be 2, 4, 8, or 16")),
    }
}

#[derive(Serialize)]
pub struct QuantizationResponse {
    status: String,
    quantized_path: String,
}

pub struct QuantizationHandler {
    model_store: ModelStore,
}

impl QuantizationHandler {
    pub fn new(model_store: ModelStore) -> Self {
        QuantizationHandler { model_store }
    }

    #[cfg(feature = "server")]
    pub fn quantize(&self, req: web::Json<QuantizationRequest>) -> Result<HttpResponse, actix_web::Error> {
        req.validate().map_err(|e| actix_web::error::ErrorBadRequest(e))?;

        let model_path = self.model_store.get_model_path(&req.model_name)
            .ok_or_else(|| QuantizationError::ModelNotFound(req.model_name.clone()))?;
        let bit_depth = match req.bit_depth.as_str() {
            "2" => PrecisionLevel::Bit2,
            "4" => PrecisionLevel::Bit4,
            "8" => PrecisionLevel::Bit8,
            "16" => PrecisionLevel::Bit16,
            _ => return Err(actix_web::error::ErrorBadRequest("Invalid bit depth")),
        };

        let model_data = fs::read(&model_path).map_err(QuantizationError::Io)?;
        let mut results = Vec::new();
        for (i, byte) in model_data.iter().enumerate() {
            results.push(QuantizationResult {
                token_id: i as u32,
                precision: bit_depth.clone(),
                salience_score: 0.5,
                row: i,
                role: "quantized".to_string(),
                role_confidence: 0.9,
            });
        }

        // Apply KIVI quantization
        let (grouped_keys, residual_keys, grouped_values, residual_values) = self.apply_kivi_quantization(&mut results, &bit_depth)?;

        let quantized_path = format!("quantized_{}_{}.bin", req.model_name, req.bit_depth);
        let quantized_data = bincode::serialize(&(grouped_keys, residual_keys, grouped_values, residual_values)).unwrap();
        fs::write(&quantized_path, quantized_data).map_err(QuantizationError::Io)?;
        self.model_store.vault.store(format!("quantized_{}", req.model_name), quantized_data).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        self.model_store.add_model(quantized_path.clone(), format!("{}_{}", req.model_name, req.bit_depth), "Zeta Reticula".to_string()).await;

        Ok(HttpResponse::Ok().json(QuantizationResponse {
            status: "Success".to_string(),
            quantized_path,
        }))
    }

    fn apply_kivi_quantization(&self, results: &mut Vec<QuantizationResult>, bit_depth: &PrecisionLevel) -> Result<(Array2<f16>, Array2<f32>, Array2<f16>, Array2<f32>), QuantizationError> {
        let group_size = 16;
        let token_count = results.len();
        let dim = 768; // Assuming a fixed dimension for simplicity

        let mut grouped_keys = Array2::<f16>::zeros((token_count / group_size, dim));
        let mut residual_keys = Array2::<f32>::zeros(((token_count % group_size), dim));
        for i in 0..token_count {
            let row = i / group_size;
            let col = i % group_size;
            if row < grouped_keys.dim().0 {
                for d in 0..dim {
                    grouped_keys[[row, d]] = f16::from_f32(results[i].salience_score).quantize_2bit();
                }
            } else {
                for d in 0..dim {
                    residual_keys[[col, d]] = results[i].salience_score;
                }
            }
        }

        let mut grouped_values = Array2::<f16>::zeros((token_count / group_size, dim));
        let mut residual_values = Array2::<f32>::zeros(((token_count % group_size), dim));
        for i in 0..token_count {
            let row = i / group_size;
            let col = i % group_size;
            if row < grouped_values.dim().0 {
                for d in 0..dim {
                    grouped_values[[row, d]] = f16::from_f32(results[i].salience_score).quantize_2bit();
                }
            } else {
                for d in 0..dim {
                    residual_values[[col, d]] = results[i].salience_score;
                }
            }
        }

        Ok((grouped_keys, residual_keys, grouped_values, residual_values))
    }

    #[cfg(feature = "wasm")]
    #[wasm_bindgen]
    pub fn quantize_wasm(model_name: String, bit_depth: String) -> js_sys::Promise {
        future_to_promise(async move {
            let model_store = ModelStore::new().await;
            let handler = QuantizationHandler::new(model_store);
            let req = QuantizationRequest { model_name, bit_depth };
            if let Err(e) = req.validate() {
                return Err(js_sys::Error::new(&e.to_string()).into());
            }

            let model_path = handler.model_store.get_model_path(&req.model_name)
                .ok_or_else(|| js_sys::Error::new(&QuantizationError::ModelNotFound(req.model_name).to_string()))?;
            let model_data = wasm_bindgen::JsValue::from_str(&model_path);
            let bit_depth = match req.bit_depth.as_str() {
                "2" => PrecisionLevel::Bit2,
                "4" => PrecisionLevel::Bit4,
                "8" => PrecisionLevel::Bit8,
                "16" => PrecisionLevel::Bit16,
                _ => return Err(js_sys::Error::new("Invalid bit depth").into()),
            };

            let model_data_vec = js_sys::Uint8Array::new(&model_data).to_vec();
            let mut results = Vec::new();
            for (i, byte) in model_data_vec.iter().enumerate() {
                results.push(QuantizationResult {
                    token_id: i as u32,
                    precision: bit_depth.clone(),
                    salience_score: *byte as f32,
                    row: i,
                    role: "quantized".to_string(),
                    role_confidence: 0.9,
                });
            }

            let (grouped_keys, residual_keys, grouped_values, residual_values) = handler.apply_kivi_quantization(&mut results, &bit_depth)?;
            let quantized_data = bincode::serialize(&(grouped_keys, residual_keys, grouped_values, residual_values)).unwrap();
            let quantized_path = format!("quantized_{}_{}.bin", req.model_name, req.bit_depth);
            handler.model_store.vault.store(format!("quantized_{}", req.model_name), quantized_data.clone()).await.map_err(|e| js_sys::Error::new(&e.to_string()))?;
            handler.model_store.add_model(quantized_path.clone(), format!("{}_{}", req.model_name, req.bit_depth), "Zeta Reticula".to_string()).await;

            Ok(js_sys::Array::of1(&JsValue::from_serde(&QuantizationResponse {
                status: "Success".to_string(),
                quantized_path,
            }).unwrap()).into())
        })
    }
}