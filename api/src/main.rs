// Copyright 2025 ZETA RETICULA INC
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


mod inference_handler;
mod quantization_handler;
mod model_store;
mod zeta_vault;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, Error, dev::ServiceRequest};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use jsonwebtoken::{decode, encode, Algorithm, Validation, DecodingKey, EncodingKey, Header};
use oauth2::{ClientId, ClientSecret, AuthUrl, TokenUrl, basic::BasicClient};
use serde_json::json;
use reqwest;
use log;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use lru;
use uuid;
use chrono;
use oauth2;
use stripe;
use salience_engine;
use model_store;
use zeta_vault;

//AppState defines the state of the application in actix-web using actix-web's Data struct
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub oauth_client: BasicClient,
    pub stripe_client: stripe::Client,
    pub model_store: model_store::ModelStore,
    pub zeta_vault: zeta_vault::ZetaVault,
}

//Main function for actix-web server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //Initialize logging
    env_logger::init();

    //Connect to database
    let db_url = "postgres://postgres:postgres@localhost:5432/zeta";
    //Create database pool
    let db_pool = PgPool::connect(db_url).await.unwrap();

    //Initialize oauth client
    let oauth_client = BasicClient::new(
        ClientId::new("your-client-id".to_string()),
        Some(ClientSecret::new("your-client-secret".to_string())),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()),
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())),
    );

    let stripe_client = stripe::Client::new("your-stripe-api-key".to_string());

    let model_store = model_store::ModelStore::new();
    let zeta_vault = zeta_vault::ZetaVault::new();

    let app_state = web::Data::new(AppState {
        db: db_pool,
        oauth_client,
        stripe_client,
        model_store,
        zeta_vault,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(auth_callback)
            .service(protected_route)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

//Lazy static variables
static OPA_CACHE: Lazy<Mutex<lru::LruCache<String, bool>>> = Lazy::new(|| Mutex::new(lru::LruCache::new(100)));
static USAGE_TRACKER: Lazy<Mutex<Vec<(String, u32)>>> = Lazy::new(|| Mutex::new(Vec::new()));

//Claims struct for JWT
#[derive(Deserialize, Serialize)]
struct Claims {
    sub: String,
    exp: i64,
    attributes: serde_json::Value,
}

//AuthRequest struct for OAuth callback
#[derive(Deserialize)]
struct AuthRequest {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct ResourceAttributes {
    resource_type: String,
    required_plan: String,
}

#[derive(Serialize)]
struct AuthResponse {
    status: String,
    access_token: String,
    attributes: serde_json::Value,
    upgrade_prompt: Option<String>,
}

//Auth callback handler
async fn auth_callback(
    req: web::Json<AuthRequest>,
    data: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    //Exchange code for token
    let client = &data.oauth_client;
    //client exchange
    let token = client
        .exchange_code(oauth2::AuthorizationCode::new(req.code.clone())) //clone code
        .request_async(oauth2::reqwest::async_http_client) //async http client
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    let user_info = reqwest::Client::new()
        .get("https://www.googleapis.com/userinfo/v2/me")
        .bearer_auth(token.access_token().secret())
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let email = user_info["email"].as_str().unwrap_or("unknown");
    let user_id = uuid::Uuid::new_v4().to_string();
    let row = sqlx::query("SELECT subscription_plan, subscription_status, expires_at FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(&data.db)
        .await?;

    let attributes = match row {
        Some(r) => json!({
            "user_id": user_id,
            "email": email,
            "subscription_plan": r.get::<String, _>("subscription_plan"),
            "subscription_status": r.get::<String, _>("subscription_status"),
            "expires_at": r.get::<chrono::DateTime<chrono::Utc>, _>("expires_at").to_rfc3339(),
        }),
        None => json!({
            "user_id": user_id,
            "email": email,
            "subscription_plan": "basic",
            "subscription_status": "pending",
            "expires_at": chrono::Utc::now().to_rfc3339(),
        }),
    };

    let jwt = encode(
        &Header::default(),
        &Claims {
            sub: user_id.clone(),
            exp: chrono::Utc::now().timestamp() + 3600,
            attributes: attributes.clone(),
        },
        &EncodingKey::from_secret("your-jwt-secret".as_ref()),
    )?;

    let upgrade_prompt = check_upgrade_prompt(&user_id, &attributes);
    Ok(web::Json(AuthResponse {
        status: "success".to_string(),
        access_token: jwt,
        attributes,
        upgrade_prompt,
    }))
}

fn check_upgrade_prompt(user_id: &str, attributes: &serde_json::Value) -> Option<String> {
    if attributes["subscription_plan"] == "basic" || attributes["subscription_plan"] == "pro" {
        let mut tracker = USAGE_TRACKER.lock().unwrap();
        let usage = tracker.iter_mut().find(|(id, _)| id == user_id).map(|(_, count)| {
            *count += 1;
            *count
        }).unwrap_or_else(|| {
            tracker.push((user_id.to_string(), 1));
            1
        });

        if usage > 50 && attributes["subscription_plan"] == "basic" {
            return Some("Upgrade to Pro for more requests!".to_string());
        } else if usage > 200 && attributes["subscription_plan"] == "pro" {
            return Some("Upgrade to Enterprise for unlimited inferences!".to_string());
        }
    }
    None
}

async fn authorize(
    req: ServiceRequest,
    srv: actix_web::dev::Service<actix_web::dev::ServiceRequest>,
) -> Result<actix_web::dev::ServiceResponse, Error> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        let token = auth_header.to_str().unwrap_or("").replace("Bearer ", "");
        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret("your-jwt-secret".as_ref()),
            &Validation::new(Algorithm::HS256),
        )?;

        let attributes = token_data.claims.attributes;
        let resource_attrs = req
            .match_info()
            .get("resource_attrs")
            .and_then(|s| serde_json::from_str::<ResourceAttributes>(s).ok())
            .unwrap_or(ResourceAttributes {
                resource_type: "unknown".to_string(),
                required_plan: "basic".to_string(),
            });

        let cache_key = format!("{:?}-{:?}", attributes, resource_attrs);
        let mut cache = OPA_CACHE.lock().unwrap();
        if let Some(allow) = cache.get(&cache_key) {
            if !*allow {
                return Err(actix_web::error::ErrorForbidden("Access denied by cached policy"));
            }
        } else {
            let opa_client = reqwest::Client::new();
            let opa_response = opa_client
                .post("http://opa:8181/v1/data/authz/allow")
                .json(&json!({
                    "attributes": attributes,
                    "resource_attrs": resource_attrs,
                }))
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            let allow = opa_response["result"]["allow"].as_bool().unwrap_or(false);
            cache.put(cache_key, allow);
            if !allow {
                return Err(actix_web::error::ErrorForbidden("Access denied by policy"));
            }
        }

        req.extensions_mut().insert(attributes);
        srv.call(req).await
    } else {
        Err(actix_web::error::ErrorUnauthorized("No token provided"))
    }
}

async fn protected_route(
    data: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> Result<impl Responder, Error> {
    let attributes = req.extensions().get::<serde_json::Value>().unwrap();
    Ok(HttpResponse::Ok().json(json!({
        "message": "Protected resource accessed",
        "attributes": attributes
    })))
}

struct AppState {
    db: PgPool,
    oauth_client: BasicClient,
    stripe_client: stripe::Client,
    model_store: model_store::ModelStore,
    zeta_vault: zeta_vault::ZetaVault,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let db_url = std::env::var("NEON_CONNECTION_STRING").expect("NEON_CONNECTION_STRING must be set");
    let db = PgPool::connect(&db_url).await.expect("Failed to connect to Neon");
    let stripe_secret = std::env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY must be set");
    let stripe_client = stripe::Client::new(stripe_secret);
    let google_client_id = ClientId::new(std::env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"));
    let google_client_secret = ClientSecret::new(std::env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET must be set"));
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap();
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap();
    let oauth_client = BasicClient::new(google_client_id, Some(google_client_secret), auth_url, Some(token_url));
    let model_store = model_store::ModelStore::new();
    let zeta_vault = zeta_vault::ZetaVault::new();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db: db.clone(),
                oauth_client: oauth_client.clone(),
                stripe_client: stripe_client.clone(),
                model_store: model_store.clone(),
                zeta_vault: zeta_vault.clone(),
            }))
            .wrap(actix_web::middleware::Logger::new("%a %{User-Agent}i %r %s %b %Dms"))
            .service(web::resource("/auth/callback").to(auth_callback))
            .service(web::resource("/protected/{resource_attrs}").wrap(authorize).to(protected_route))
            .service(web::resource("/subscribe/{resource_attrs}").wrap(authorize).to(subscribe))
    })
    .bind(("0.0.0.0", 8080))?
    .workers(4)
    .run()
    .await
}