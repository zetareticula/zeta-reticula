use mlua::prelude::*;
use crate::inference_handler::InferenceHandler;
use crate::quantization_handler::QuantizationHandler;
use crate::model_store::ModelStore;
use crate::zeta_vault::{ZetaVault, VaultConfig, KVCache, CacheLayer};
use ndarray::Array2;
use half::f16;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Creates a Lua module for Zeta Reticula with inference and quantization handlers
/// # Arguments:
/// * `lua`: The Lua state to register the module in
/// # Returns:
/// * A Lua table containing the inference and quantization handlers
/// # Errors:
/// 
/// If the Lua state cannot be created or if there are errors in the handlers, it returns an error

pub fn create_lua_module(lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
    let globals = lua.globals();

    let inference_handler = lua.create_table()?;
    inference_handler.set("new", lua.create_function(|_lua, ()| {
        let model_store = ModelStore::new().await;
        let handler = InferenceHandler::new(model_store).await;
        Ok(mlua::Value::Table(inference_handler.clone()))
    })?)?;
    inference_handler.set("infer", lua.create_function(|_lua, (handler, input, model_name, precision): (mlua::Table, String, String, String)| {
        let req = crate::inference_handler::InferenceRequest {
            input,
            model_name,
            precision,
        };
        if let Err(e) = req.validate() {
            return Err(mlua::Error::RuntimeError(e.to_string()));
        }
        let resp = handler.get::<_, InferenceHandler>("handler")?.infer(&web::Json(req)).map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        let json = resp.json::<crate::inference_handler::InferenceResponse>().map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        Ok(mlua::Value::Table(lua.create_table_from([
            ("text", mlua::Value::String(lua.create_string(&json.text)?)),
            ("tokens_processed", mlua::Value::Integer(json.tokens_processed as i64)),
            ("latency_ms", mlua::Value::Number(json.latency_ms)),
        ])?))
    })?)?;

    let quantization_handler = lua.create_table()?;
    quantization_handler.set("new", lua.create_function(|_lua, ()| {
        let model_store = ModelStore::new().await;
        let handler = QuantizationHandler::new(model_store);
        Ok(mlua::Value::Table(quantization_handler.clone()))
    })?)?;
    quantization_handler.set("quantize", lua.create_function(|_lua, (handler, model_name, bit_depth): (mlua::Table, String, String)| {
        let req = crate::quantization_handler::QuantizationRequest {
            model_name,
            bit_depth,
        };
        if let Err(e) = req.validate() {
            return Err(mlua::Error::RuntimeError(e.to_string()));
        }
        let resp = handler.get::<_, QuantizationHandler>("handler")?.quantize(&web::Json(req)).map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        let json = resp.json::<crate::quantization_handler::QuantizationResponse>().map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        Ok(mlua::Value::String(lua.create_string(&json.quantized_path)?))
    })?)?;

    globals.set("zeta_reticula", lua.create_table_from([
        ("inference", mlua::Value::Table(inference_handler)),
        ("quantization", mlua::Value::Table(quantization_handler)),
    ])?)?;

    Ok(mlua::Value::Nil)
}


/// Error type for Lua module creation
/// This error type is used to handle errors that occur during the creation of the Lua module.
/// #[derive(Debug, Error)]
pub enum LuaModuleError {
    #[error("Failed to create Lua state: {0}")]
    LuaStateCreation(#[from] mlua::Error),
    #[error("Handler error: {0}")]
    HandlerError(String),
}

impl From<mlua::Error> for LuaModuleError {
    fn from(err: mlua::Error) -> Self {
        LuaModuleError::LuaStateCreation(err)
    }
}

pub fn initialize_lua_module() -> Result<mlua::Value, LuaModuleError> {

    if !cfg!(feature = "lua") {
        return Err(LuaModuleError::HandlerError("Lua feature is not enabled".to_string()));
    }

    if cfg!(feature = "enterprise") {
        log::warn!("Lua module is not available in enterprise mode");
        return Err(LuaModuleError::HandlerError("Lua module is not available in enterprise mode".to_string()));
    }

    for arg in std::env::args() {
        if arg == "--lua" {
            log::info!("Lua module enabled");
        }
    }

    let lua = mlua::Lua::new();
    create_lua_module(&lua).map_err(LuaModuleError::from)
    Ok(mlua::Value::Nil)
}
