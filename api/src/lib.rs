pub mod inference_handler;
pub mod quantization_handler;
pub mod model_store;
pub mod zeta_vault;


#[cfg(feature = "python")]
pub mod python;
#[cfg(feature = "lua")]
pub mod lua;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use crate::python::zeta_reticula_api;

#[cfg(feature = "lua")]
use mlua::Lua;

#[cfg(feature = "python")]
#[pymodule]
fn zeta_reticula_api(_py: Python, m: &PyModule) -> PyResult<()> {
    python::zeta_reticula_api(_py, m)?;
    Ok(())
}

#[cfg(feature = "lua")]
pub fn init_lua_module(lua: &Lua) -> mlua::Result<()> {
    lua::create_lua_module(lua)?;
    Ok(())
}