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

use shared::{PrecisionLevel, QuantizationResult};
use serde::{Serialize, Deserialize};
use ndarray::{Array1, Array2, s};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct Predictor {
    threshold: f32,
}

impl Predictor {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }

    pub fn predict_active_neurons(&self, preactivations: &Array1<f32>) -> Vec<bool> {
        preactivations.iter().map(|&x| x > self.threshold).collect()
    }
}

//
#[derive(Serialize, Deserialize)]
pub struct Model {
    matrix: Array2<f32>,
    pointers: Vec<usize>,
    bias: Array1<f32>,
    num_used: usize,
    last_k_active: Vec<usize>,
    precision_config: Vec<PrecisionLevel>,
    predictor: Predictor,
    chunk_size: usize,
    d_model: usize,
}

impl Model {
    pub fn new(size: usize, quantization_results: &[QuantizationResult]) -> Self {
        let d_model = 768;
        let rows = size / d_model;
        Self {
            matrix: Array2::zeros((rows, 2 * d_model)),
            pointers: vec![0; rows],
            bias: Array1::zeros(rows),
            num_used: 0,
            last_k_active: vec![],
            precision_config: quantization_results.iter().map(|r| r.precision.clone()).collect(),
            predictor: Predictor::new(0.1),
            chunk_size: 32 * 1024,
            d_model,
        }
    }

    pub async fn load_from_flash(&mut self, file_path: &str) -> tokio::io::Result<()> {
        let file = File::open(file_path).await?;
        let mut reader = BufReader::new(file);
        let bytes_per_row = 2 * self.d_model * 4;
        let total_rows = self.matrix.nrows();
        let mut buffer = vec![0u8; bytes_per_row];

        for row in 0..total_rows {
            if reader.read_exact(&mut buffer).await.is_err() {
                break;
            }
            for col in 0..2 * self.d_model {
                let start = col * 4;
                let val = f32::from_le_bytes([
                    buffer[start],
                    buffer[start + 1],
                    buffer[start + 2],
                    buffer[start + 3],
                ]);
                self.matrix[[row, col]] = val;
            }
        }
        Ok(())
    }

    pub fn quantize(&mut self, quantization_results: &[QuantizationResult]) {
        self.precision_config = quantization_results.iter().map(|r| r.precision.clone()).collect();
        for (i, precision) in self.precision_config.iter().enumerate().take(self.matrix.nrows()) {
            let scale = match precision {
                PrecisionLevel::Bit4 => 0.0625,
                PrecisionLevel::Bit8 => 0.00390625,
                PrecisionLevel::Bit16 => 1.0,
            };
            self.matrix.slice_mut(s![i, ..]).mapv_inplace(|x| x * scale);
        }
    }

    pub fn delete_neurons(&mut self, inactive: &[usize]) {
        for &neuron in inactive {
            if let Some(idx) = self.pointers.iter().position(|&p| p == neuron) {
                if idx < self.num_used {
                    if let Some(last) = self.last_k_active.pop() {
                        self.matrix.row_mut(idx).assign(&self.matrix.row(self.num_used - 1));
                        self.pointers[idx] = self.pointers[self.num_used - 1];
                        self.bias[idx] = self.bias[self.num_used - 1];
                    }
                    self.num_used -= 1;
                }
            }
        }
    }

    pub fn add_neurons(&mut self, neurons: &[usize], weights: &[f32], biases: &[f32]) {
        let available = self.matrix.nrows() - self.num_used;
        let count = neurons.len().min(available);
        if count == 0 { return; }

        let new_weights = Array2::from_shape_vec((count, self.matrix.ncols()), weights.to_vec())
            .expect("Invalid shape for new weights");

        self.matrix.slice_mut(s![self.num_used..self.num_used + count, ..])
            .assign(&new_weights);

        for i in 0..count {
            self.pointers[self.num_used + i] = neurons[i];
            self.bias[self.num_used + i] = biases[i];
        }
        self.num_used += count;
    }

    pub fn compute_ffn(&self, input: &Array1<f32>) -> Array1<f32> {
        let proj = input.dot(&self.matrix.slice(s![..self.num_used, ..self.d_model]));
        let active = self.predictor.predict_active_neurons(&proj);
        let mut output = Array1::zeros(proj.len());

        for (i, &flag) in active.iter().enumerate() {
            if flag {
                let up = proj[i] + self.bias[i];
                let down = self.matrix.slice(s![i, self.d_model..]).dot(&Array1::ones(self.d_model));
                output[i] = (up * down).max(0.0);
            }
        }

        output
    }
}
