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

// Feature-gated modules
#[cfg(feature = "server")]
pub mod inference_handler;
#[cfg(feature = "server")]
pub mod quantization_handler;
#[cfg(feature = "server")]
pub mod model_store;
#[cfg(feature = "server")]
pub mod zeta_vault;
#[cfg(feature = "lua")]
pub mod lua;
#[cfg(feature = "python")]
pub mod python;
#[cfg(feature = "server")]
pub mod zeta_vault_synergy;

// Server-specific imports and state
#[cfg(feature = "server")]
use actix_web::{web, App, HttpServer, Responder, HttpResponse, Error};
#[cfg(feature = "server")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::PgPool;
#[cfg(feature = "server")]
use jsonwebtoken::{decode, encode, Algorithm, Validation, DecodingKey, EncodingKey, Header};

// Define the application state (server only)
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub oauth_client: oauth2::basic::BasicClient,
}

#[cfg(feature = "server")]
impl AppState {
    pub fn new(db_pool: PgPool, oauth_client: oauth2::basic::BasicClient) -> Self {
        AppState { db_pool, oauth_client }
    }
}
