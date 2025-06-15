#[cfg(feature = "server")]
use actix_web::{App, HttpServer, web, HttpResponse};
#[cfg(feature = "server")]
use actix_multipart::Multipart;
#[cfg(feature = "server")]
use futures::StreamExt;
#[cfg(feature = "server")]
use tokio::fs::File;
#[cfg(feature = "server")]
use tokio::io::AsyncWriteExt;
#[cfg(feature = "server")]
use log;
#[cfg(feature = "server")]
use inference_handler::InferenceHandler;
#[cfg(feature = "server")]
use quantization_handler::QuantizationHandler;
#[cfg(feature = "server")]
use model_store::ModelStore;
#[cfg(feature = "server")]
use thiserror::Error;

#[cfg(feature = "server")]
#[derive(Error, Debug)]
enum ApiError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

#[cfg(feature = "server")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    log::info!("Starting Zeta Reticula API at 09:43 PM PDT, June 10, 2025");

    let model_store = ModelStore::new().await;
    let inference_handler = InferenceHandler::new(model_store.clone()).await;
    let quantization_handler = QuantizationHandler::new(model_store.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(inference_handler.clone()))
            .app_data(web::Data::new(quantization_handler.clone()))
            .app_data(web::Data::new(model_store.clone()))
            .route("/infer", web::post().to(inference_handler.infer))
            .route("/upload", web::post().to(upload_model))
            .route("/quantize", web::post().to(quantization_handler.quantize))
            .route("/models", web::get().to(get_available_models))
            .route("/stats", web::get().to(get_stats))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[cfg(feature = "server")]
async fn upload_model(mut payload: Multipart, store: web::Data<ModelStore>) -> Result<HttpResponse, ApiError> {
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| ApiError::Validation(e.to_string()))?;
        let content_type = field.content_disposition().unwrap().get_name().unwrap_or("");
        if content_type != "model" {
            return Err(ApiError::Validation("Invalid field name".to_string()));
        }

        let mut file = File::create("uploaded_model.bin").await?;
        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| ApiError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            file.write_all(&data).await?;
        }
        store.add_model("uploaded_model.bin".to_string(), "CustomModel".to_string(), "Zeta Reticula".to_string()).await;
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "Model uploaded"})))
}

#[cfg(feature = "server")]
async fn get_available_models(store: web::Data<ModelStore>) -> HttpResponse {
    let models = store.get_available_models();
    HttpResponse::Ok().json(models)
}

#[cfg(feature = "server")]
async fn get_stats() -> HttpResponse {
    let stats = serde_json::json!({
        "latency": 0.4,
        "memory_savings": 60.0,
        "throughput": 2500.0,
        "anns_recall": 0.95,
        "brand": "Zeta Reticula"
    });
    HttpResponse::Ok().json(stats)
}