pub mod inference_handler;
pub mod quantization_handler;
pub mod model_store;
pub mod zeta_vault;
pub mod lua;
pub mod python;

use actix_web::{web, App, HttpServer, Responder, HttpResponse, Error};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use jsonwebtoken::{decode, encode, Algorithm, Validation, DecodingKey, EncodingKey, Header};


// Define the application state
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub oauth_client: oauth2::basic::BasicClient,
}

impl AppState {
    pub fn new(db_pool: PgPool, oauth_client: oauth2::basic::BasicClient) -> Self {
        AppState { db_pool, oauth_client }
    }
}





