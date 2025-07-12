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

use pyo3::prelude::*;
use crate::inference_handler::InferenceHandler;
use crate::quantization_handler::QuantizationHandler;
use crate::model_store::ModelStore;
use crate::zeta_vault::{ZetaVault, VaultConfig, KVCache, CacheLayer};
use ndarray::Array2;
use half::f16;
use std::sync::Arc;
use std::sync::Mutex;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, Error};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::Validate;

#[pyclass]
#[derive(Debug, Serialize, Deserialize)]
struct NeuronMatrix {
    matrix: Array2<f16>,
    pointers: Vec<usize>,
    bias: Vec<f32>,
}

#[pymethods]
impl NeuronMatrix {
    #[new]
    fn new(matrix: Vec<Vec<f16>>, pointers: Vec<usize>, bias: Vec<f32>) -> PyResult<Self> {
        let array_matrix = Array2::from_shape_vec((matrix.len(), matrix[0].len()), matrix.into_iter().flatten().collect())
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(NeuronMatrix {
            matrix: array_matrix,
            pointers,
            bias,
        })
    }
}


#[pyclass]
#[derive(Debug, Serialize, Deserialize)]
struct CompactionRequest {
    model_id: String,
    level: usize,
    data: Vec<u8>,
}

#[pyclass]
struct PyInferenceHandler {
    handler: InferenceHandler,
}

#[pymethods]
impl PyInferenceHandler {
    #[new]
    async fn new() -> PyResult<Self> {
        let model_store = ModelStore::new().await;
        let handler = InferenceHandler::new(model_store).await;
        Ok(PyInferenceHandler { handler })
    }

    fn infer(&self, input: String, model_name: String, precision: String) -> PyResult<(String, usize, f64)> {
        let req = crate::inference_handler::InferenceRequest {
            input,
            model_name,
            precision,
            user_id: todo!(),
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