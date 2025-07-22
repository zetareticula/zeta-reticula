// Copyright 2025 ZETA RETICULA
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use actix_web::{web, HttpResponse, Error};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Header, EncodingKey};
use crate::api::petri_engine::{PetriPlace, PetriTransition, PetriEngine};
use crate::attention_store::AttentionStore;
use crate::agentflow::AgentFlow;
use crate::zeta_vault_synergy::ZetaVaultSynergy;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct AuthRequest {
    email: String,
    password: String, // In production, use OAuth or hashed passwords
}

#[derive(Debug, Serialize, Deserialize)]
struct QuantizeRequest {
    model_id: String,
    bit_width: u8,
    lora_enabled: bool,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(web::resource("/auth").route(web::post().to(auth_handler)))
            .service(web::resource("/quantize").route(web::post().to(quantize_handler)))
            .service(web::resource("/inference").route(web::post().to(inference_handler)))
            .service(web::resource("/agent/status").route(web::get().to(agent_status_handler))),
    );
}

async fn auth_handler(
    req: web::Json<AuthRequest>,
    vault: web::Data<Arc<ZetaVaultSynergy>>,
) -> Result<HttpResponse, Error> {
    // Basic validation (enhance with regex and sanitization in production)
    if req.email.is_empty() || req.password.is_empty() {
        return Ok(HttpResponse::BadRequest().body("Invalid credentials"));
    }

    // Mock token generation (replace with real auth logic and secret key)
    let token = encode(
        &jsonwebtoken::Header::default(),
        &jsonwebtoken::EncodingKey::from_secret("secret".as_ref()),
        &Header::default(),
    ).map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    vault.store_token(&req.email, &token).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"token": token})))
}

async fn quantize_handler(
    req: web::Json<QuantizeRequest>,
    store: web::Data<Arc<AttentionStore>>,
    engine: web::Data<Arc<PetriEngine>>,
) -> Result<HttpResponse, Error> {
    if req.bit_width > 8 || req.bit_width < 2 {
        return Ok(HttpResponse::BadRequest().body("Bit width must be 2, 4, or 8"));
    }

    let kv_cache = store.prefill(req.model_id.clone(), vec![1, 2, 3]).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    engine.update_kv_cache(&req.model_id, &kv_cache, req.bit_width, req.lora_enabled).await;
    Ok(HttpResponse::Ok().body("Quantization initiated"))
}

async fn inference_handler(
    model_id: web::Path<String>,
    store: web::Data<Arc<AttentionStore>>,
    engine: web::Data<Arc<PetriEngine>>,
) -> Result<HttpResponse, Error> {
    let (token, kv_cache) = store.decode(model_id.into_inner(), 1, vec![]).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    engine.update_kv_cache(&model_id, &kv_cache, 8, false).await;
    Ok(HttpResponse::Ok().json(serde_json::json!({"token": token})))
}

async fn agent_status_handler(
    engine: web::Data<Arc<PetriEngine>>,
) -> Result<HttpResponse, Error> {
    let status = engine.get_agent_status();
    Ok(HttpResponse::Ok().json(status))
}