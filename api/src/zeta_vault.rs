use serde_json::Value;
use std::collections::HashMap;
use actix_web::{web, Error, Responder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use thiserror::Error;
use validator::Validate;
use zeta_vault::ZetaVault;


#[derive(Deserialize, Serialize, Validate)]
pub struct VaultConfig {
    #[validate(length(min = 1, message = "Vault name is required"))]
    vault_name: String,
    #[validate(length(min = 1, message = "User ID is required"))]
    user_id: String,
}

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Invalid attributes format")]
    InvalidAttributesFormat,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct UserAttributes {
    #[validate(length(min = 1, message = "User ID is required"))]
    user_id: String,
    #[validate]
    attributes: Value, // Assuming attributes are in JSON format
}



#![derive(Deserialize, Serialize, Validate)]
pub struct ZetaVault {
    store: HashMap<String, Value>,
}

impl ZetaVault {
    pub fn new() -> Self {
        ZetaVault { store: HashMap::new() }
    }

    pub fn get_user_attributes(&self, user_id: &str) -> Option<Value> {
        self.store.get(user_id).cloned()
    }

    pub fn set_user_attributes(&mut self, user_id: String, attributes: Value) {
        self.store.insert(user_id, attributes);
    }
}



/// This function handles the request to get user attributes from the vault
/// # Arguments:
/// * `req`: The request containing the user ID
/// # Returns:
/// * A response containing the user attributes or an error
async fn get_user_attributes(
    req: web::Json<UserAttributes>,
    vault: web::Data<ZetaVault>,
) -> Result<impl Responder, VaultError> {
    // Validate the request
    req.validate().map_err(|_| VaultError::InvalidAttributesFormat)?;

    // Get user attributes from the vault
    match vault.get_user_attributes(&req.user_id) {
        Some(attributes) => Ok(web::Json(attributes)),
        None => Err(VaultError::UserNotFound(req.user_id.clone())),
    }
}

/// This function handles the request to set user attributes in the vault
/// # Arguments:
/// * `req`: The request containing the user ID and attributes
/// # Returns:
/// * A response indicating success or an error

async fn set_user_attributes(
    req: web::Json<UserAttributes>,
    vault: web::Data<ZetaVault>,
) -> Result<impl Responder, VaultError> {
    // Validate the request
    req.validate().map_err(|_| VaultError::InvalidAttributesFormat)?;

    // Set user attributes in the vault
    vault.set_user_attributes(req.user_id.clone(), req.attributes.clone());

    Ok(web::Json(json!({"status": "success"})))
}