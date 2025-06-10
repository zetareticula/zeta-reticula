use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use actix_multipart::Multipart;
use futures::StreamExt;
use serde::{Serialize, Deserialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Serialize)]
struct Stats {
    latency: f32,
    memory_savings: f32,
    throughput: f32,
    anns_recall: f32,
}

#[get("/models")]
async fn get_models() -> impl Responder {
    let models = vec![
        "Mistral-7B".to_string(),
        "OPT-6.7B".to_string(),
        "LLaMA-13B".to_string(),
    ];
    HttpResponse::Ok().json(models)
}

#[post("/upload")]
async fn upload_model(mut payload: Multipart) -> impl Responder {
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(_) => return HttpResponse::BadRequest().body("Upload failed"),
        };

        let mut file = File::create("uploaded_model.bin").await.unwrap();
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            file.write_all(&data).await.unwrap();
        }
    }

    #[derive(Serialize)]
    struct UploadResponse {
        model_name: String,
    }
    HttpResponse::Ok().json(UploadResponse {
        model_name: "CustomModel".to_string(),
    })
}

#[get("/stats")]
async fn get_stats() -> impl Responder {
    let stats = Stats {
        latency: 0.4,
        memory_savings: 60.0,
        throughput: 2500.0,
        anns_recall: 0.95,
    };
    HttpResponse::Ok().json(stats)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_models)
            .service(upload_model)
            .service(get_stats)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}