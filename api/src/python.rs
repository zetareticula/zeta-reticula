use pyo3::prelude::*;
use crate::inference_handler::InferenceHandler;
use crate::quantization_handler::QuantizationHandler;
use crate::model_store::ModelStore;
use crate::zeta_vault::{ZetaVault, VaultConfig, KVCache, CacheLayer};
use ndarray::Array2;
use half::f16;

#[pyclass]
struct PyInferenceHandler {
    handler: InferenceHandler,
}

#[pymethods]
impl PyInferenceHandler {
    #[new]
    fn new() -> PyResult<Self> {
        let model_store = ModelStore::new().await;
        let handler = InferenceHandler::new(model_store).await;
        Ok(PyInferenceHandler { handler })
    }

    fn infer(&self, input: String, model_name: String, precision: String) -> PyResult<(String, usize, f64)> {
        let req = crate::inference_handler::InferenceRequest {
            input,
            model_name,
            precision,
        };
        if let Err(e) = req.validate() {
            return Err(pyo3::exceptions::PyValueError::new_err(e.to_string()));
        }
        let resp = self.handler.infer(&web::Json(req)).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let json = resp.json::<crate::inference_handler::InferenceResponse>().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok((json.text, json.tokens_processed, json.latency_ms))
    }
}

#[pyclass]
struct PyQuantizationHandler {
    handler: QuantizationHandler,
}

#[pymethods]
impl PyQuantizationHandler {
    #[new]
    fn new() -> PyResult<Self> {
        let model_store = ModelStore::new().await;
        let handler = QuantizationHandler::new(model_store);
        Ok(PyQuantizationHandler { handler })
    }

    fn quantize(&self, model_name: String, bit_depth: String) -> PyResult<String> {
        let req = crate::quantization_handler::QuantizationRequest {
            model_name,
            bit_depth,
        };
        if let Err(e) = req.validate() {
            return Err(pyo3::exceptions::PyValueError::new_err(e.to_string()));
        }
        let resp = self.handler.quantize(&web::Json(req)).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let json = resp.json::<crate::quantization_handler::QuantizationResponse>().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(json.quantized_path)
    }
}

#[pymodule]
fn zeta_reticula_api(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyInferenceHandler>()?;
    m.add_class::<PyQuantizationHandler>()?;
    Ok(())
}