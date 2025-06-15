use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use salience_engine::quantizer::{PrecisionLevel, QuantizationResult};
use std::fs;
use model_store::ModelStore;
use thiserror::Error;
use validator::Validate;

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
        "4" | "8" | "16" => Ok(()),
        _ => Err(validator::ValidationError::new("Bit depth must be 4, 8, or 16")),
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

    pub fn quantize(&self, req: web::Json<QuantizationRequest>) -> Result<HttpResponse, actix_web::Error> {
        req.validate().map_err(|e| actix_web::error::ErrorBadRequest(e))?;

        let model_path = self.model_store.get_model_path(&req.model_name)
            .ok_or_else(|| QuantizationError::ModelNotFound(req.model_name.clone()))?;
        let bit_depth = match req.bit_depth.as_str() {
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
                salience_score: 0.5, // Mock
                row: i,
                role: "quantized".to_string(),
                role_confidence: 0.9,
            });
        }

        let quantized_path = format!("quantized_{}_{}.bin", req.model_name, req.bit_depth);
        fs::write(&quantized_path, model_data).map_err(QuantizationError::Io)?;
        self.model_store.add_model(quantized_path.clone(), format!("{}_{}", req.model_name, req.bit_depth), "Zeta Reticula".to_string()).await;

        Ok(HttpResponse::Ok().json(QuantizationResponse {
            status: "Success".to_string(),
            quantized_path,
        }))
    }
}