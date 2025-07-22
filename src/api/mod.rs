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

pub mod routes;
pub mod petri_engine;

use actix_web::{web, App, HttpServer, middleware};
use actix_web::dev::Server;
use actix_web::http::header;
use std::sync::Arc;
use thiserror::Error;
use crate::attention_store::AttentionStore;
use crate::agentflow::AgentFlow;
use crate::zeta_vault_synergy::ZetaVaultSynergy;
use std::fs;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Internal server error: {0}")]
    Internal(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("SSL configuration error: {0}")]
    Ssl(String),
}

pub struct ApiServer {
    attention_store: Arc<AttentionStore>,
    agent_flow: Arc<AgentFlow>,
    vault: Arc<ZetaVaultSynergy>,
}

impl ApiServer {
    pub fn new(attention_store: Arc<AttentionStore>, agent_flow: Arc<AgentFlow>, vault: Arc<ZetaVaultSynergy>) -> Self {
        ApiServer {
            attention_store,
            agent_flow,
            vault,
        }
    }

    pub fn start(&self, addr: &str, cert_path: &str, key_path: &str) -> Result<Server, ApiError> {
        // Load SSL certificates and private key
        let cert_file = fs::File::open(cert_path).map_err(|e| ApiError::Ssl(format!("Failed to open cert file: {}", e)))?;
        let key_file = fs::File::open(key_path).map_err(|e| ApiError::Ssl(format!("Failed to open key file: {}", e)))?;
        let certs = certs(&mut std::io::BufReader::new(cert_file)).map_err(|e| ApiError::Ssl(format!("Failed to read certs: {}", e)))?
            .into_iter().map(Certificate).collect();
        let mut keys = pkcs8_private_keys(&mut std::io::BufReader::new(key_file)).map_err(|e| ApiError::Ssl(format!("Failed to read private key: {}", e)))?;
        if keys.is_empty() {
            return Err(ApiError::Ssl("No private key found".to_string()));
        }
        let key = PrivateKey(keys.remove(0));
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| ApiError::Ssl(format!("Failed to build SSL config: {}", e)))?;

        // Configure HTTP server with HTTPS
        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(Arc::clone(&self.attention_store)))
                .app_data(web::Data::new(Arc::clone(&self.agent_flow)))
                .app_data(web::Data::new(Arc::clone(&self.vault)))
                .configure(routes::configure)
                .wrap(middleware::Logger::default())
                // Updated CSP policy
                .wrap(middleware::DefaultHeaders::new()
                    .add((header::CONTENT_SECURITY_POLICY, "default-src 'self'; script-src 'self' https://trusted.cdn.com; style-src 'self' https://trusted.cdn.com; connect-src 'self' https://api.zeta-reticula.com wss://api.zeta-reticula.com; object-src 'none'; frame-ancestors 'none'; base-uri 'self'; form-action 'self';"))
                    .add((header::STRICT_TRANSPORT_SECURITY, "max-age=31536000; includeSubDomains; preload"))
                    .add((header::X_FRAME_OPTIONS, "DENY")))
                .wrap(actix_cors::Cors::default()
                    .allowed_origin("https://app.zeta-reticula.com")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
                    .supports_credentials()
                    .max_age(3600))
        })
        .bind_rustls(addr, config)?
        .run();

        Ok(server)
    }
}

use std::io::BufReader;