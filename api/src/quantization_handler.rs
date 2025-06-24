use actix_web::{web, HttpResponse, Error};
use serde::{Deserialize, Serialize};
use salience_engine::quantizer::{SalienceQuantizer, TokenFeatures, PrecisionLevel};
use model_store::ModelStore;
use thiserror::Error;
use validator::Validate;
use crate::python::PythonEngine;
use crate::USAGE_TRACKER;
use sqlx::PgPool;
use std::sync::Mutex;
use ndarray::Array2;

#[derive(Error, Debug)]
enum QuantizationError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Python execution error: {0}")]
    Python(#[from] pyo3::PyErr),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Compaction error: {0}")]
    Compaction(String),
}

#[derive(Validate, Deserialize)]
pub struct QuantizationRequest {
    #[validate(length(min = 1, message = "Model ID is required"))]
    model_id: String,
    #[validate(custom = "validate_precision")]
    precision: String,
    user_id: String,
}

fn validate_precision(precision: &str) -> Result<(), validator::ValidationError> {
    match precision {
        "2" | "4" | "8" | "16" => Ok(()),
        _ => Err(validator::ValidationError::new("Precision must be 2, 4, 8, or 16")),
    }
}

#[derive(Serialize, Deserialize)]
pub struct QuantizationResponse {
    status: String,
    upgrade_prompt: Option<String>,
}

pub struct QuantizationHandler {
    model_store: ModelStore,
    python_engine: PythonEngine,
    db_pool: PgPool,
}

impl QuantizationHandler {
    pub fn new(model_store: ModelStore, db_pool: PgPool) -> Self {
        QuantizationHandler {
            model_store,
            python_engine: PythonEngine::new(),
            db_pool,
        }
    }

    pub async fn quantize(&self, req: web::Json<QuantizationRequest>) -> Result<HttpResponse, Error> {
        req.validate().map_err(|e| actix_web::error::ErrorBadRequest(QuantizationError::Validation(e.to_string())))?;

        let subscription = self.get_subscription_data(&req.user_id).await?;
        let is_enterprise = subscription.plan == "enterprise" && cfg!(feature = "enterprise") && subscription.status == "active";
        let usage_limit = if is_enterprise { u32::MAX } else { 20_000 };

        let model_path = self.model_store.get_model_path(&req.model_id)
            .ok_or_else(|| QuantizationError::ModelNotFound(req.model_id.clone()))?;
        let weights = self.model_store.get_model_weights(&req.model_id).await
            .ok_or_else(|| QuantizationError::ModelNotFound(req.model_id.clone()))?;

        let quantizer = SalienceQuantizer::new(0.7);
        let tokens: Vec<&str> = std::str::from_utf8(&weights)?.split_whitespace().collect();
        let token_features: Vec<TokenFeatures> = tokens.iter()
            .enumerate()
            .map(|(i, _)| TokenFeatures {
                token_id: i as u32,
                frequency: 0.5,
                sentiment_score: 0.0,
                context_relevance: 0.5,
                role: "".to_string(),
            })
            .collect();
        let (results, _) = quantizer.quantize_tokens(token_features, &req.precision);

        let precision = match req.precision.as_str() {
            "2" => PrecisionLevel::Bit2,
            "4" => PrecisionLevel::Bit4,
            "8" => PrecisionLevel::Bit8,
            "16" => PrecisionLevel::Bit16,
            _ => return Err(actix_web::error::ErrorBadRequest(QuantizationError::Validation("Invalid precision".to_string()))),
        };

        let d_model = 768;
        let mut neuron_matrix = self.model_store.get_neuron_matrix(&req.model_id).await
            .ok_or_else(|| QuantizationError::ModelNotFound(req.model_id.clone()))?;
        let new_neurons = results.iter()
            .filter(|r| r.salience_score > 0.6)
            .map(|r| {
                let weights = Array2::from_elem((1, 2 * d_model), f16::from_f32(r.salience_score));
                (r.token_id as usize, weights, 0.0)
            })
            .collect::<Vec<_>>();
        self.model_store.update_neuron_matrix(&req.model_id, new_neurons).await;

        // Enqueue compaction for updated weights
        let compaction_request = model_store::CompactionRequest {
            model_id: req.model_id.clone(),
            level: 0, // Initial level
            data: weights,
        };
        self.model_store.enqueue_compaction(compaction_request).await;

        let quantized_input = self.python_engine.execute_quantization(&format!("Model_{}", req.model_id))?;
        let status = format!("Quantized {} with precision {} and input {}, updated {} neurons", req.model_id, precision, quantized_input, neuron_matrix.num_used);

        let tokens_processed = tokens.len() as u32;
        let upgrade_prompt = self.check_usage_limit(&req.user_id, tokens_processed, "quantization", &subscription)?;

        // Error handling for compaction
        if let Err(e) = self.handle_compaction_errors().await {
            log::warn!("Compaction error: {:?}", e);
        }

        Ok(HttpResponse::Ok().json(QuantizationResponse { status, upgrade_prompt }))
    }

    async fn get_subscription_data(&self, user_id: &str) -> Result<SubscriptionData, Error> {
        let row = sqlx::query("SELECT subscription_plan, subscription_status FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.db_pool)
            .await
            .map_err(QuantizationError::Database)?;
        Ok(SubscriptionData {
            plan: row.get::<String, _>("subscription_plan"),
            status: row.get::<String, _>("subscription_status"),
        })
    }

    fn check_usage_limit(&self, user_id: &str, tokens_processed: u32, service: &str, subscription: &SubscriptionData) -> Result<Option<String>, Error> {
        let mut tracker = USAGE_TRACKER.lock().unwrap();
        let usage = tracker.iter_mut().find(|(id, _)| id == user_id).map(|(_, count)| {
            *count += tokens_processed;
            *count
        }).unwrap_or_else(|| {
            tracker.push((user_id.to_string(), tokens_processed));
            tokens_processed
        });

        let is_enterprise = subscription.plan == "enterprise" && cfg!(feature = "enterprise") && subscription.status == "active";
        let usage_limit = if is_enterprise { u32::MAX } else { 20_000 };

        Ok(if usage > usage_limit && !is_enterprise {
            Some(format!("Upgrade to Enterprise for unlimited {}!", service))
        } else {
            None
        })
    }

    async fn handle_compaction_errors(&self) -> Result<(), QuantizationError> {
        // Mock error handling for transient and execution errors
        if rand::random::<f32>() < 0.1 { // 10% chance of error
            return Err(QuantizationError::Compaction("Transient compaction failure".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct SubscriptionData {
    plan: String,
    status: String,
}