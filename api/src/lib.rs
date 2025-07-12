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


pub mod inference_handler;
pub mod quantization_handler;
pub mod model_store;
pub mod zeta_vault;
pub mod lua;
pub mod python;
pub mod zeta_vault_synergy;

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





