use mlua::prelude::*;
use crate::inference_handler::InferenceHandler;
use crate::quantization_handler::QuantizationHandler;
use crate::model_store::ModelStore;
use crate::zeta_vault::{ZetaVault, VaultConfig, KVCache, CacheLayer};
use ndarray::Array2;
use half::f16;

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