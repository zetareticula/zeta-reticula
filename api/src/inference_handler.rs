use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use reqwest;
use crate::model_store::ModelStore;
use crate::zeta_vault::{ZetaVault, VaultConfig};
use tokio::task;

#[derive(Deserialize)]
pub struct InferenceRequest {
    input: String,
    model_name: String,
    precision: String,
}

#[derive(Serialize)]
pub struct InferenceResponse {
    text: String,
    tokens_processed: usize,
    latency_ms: f64,
}

pub async fn infer(
    req: web::Json<InferenceRequest>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let req_clone = req.clone();
    let coreweave_client = data.coreweave_client.clone();

    let result = task::spawn_blocking(move || {
        // Call CoreWeave API for GPU inference
        let coreweave_url = format!("https://api.coreweave.com/inference?model={}&input={}", req_clone.model_name, req_clone.input);
        let response = reqwest::blocking::get(&coreweave_url)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
            .json::<InferenceResponse>()
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        // Store result in Neon
        let mut vault = ZetaVault::new(VaultConfig::default());
        vault.store(&req_clone.model_name, &response.text).await;

        Ok(response)
    }).await??;

    // Push to SageMaker for deployment (optional)
    let sagemaker_url = format!("https://runtime.sagemaker.us-east-1.amazonaws.com/endpoints/{}/invocations", req.model_name);
    let sagemaker_client = reqwest::Client::new();
    sagemaker_client.post(&sagemaker_url)
        .json(&result)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(result))
}

pub struct AppState {
    model_store: ModelStore,
    coreweave_client: reqwest::Client,
    // Add SageMaker client or config if needed
}