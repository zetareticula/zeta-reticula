use shared::{PrecisionLevel, QuantizationResult};
use serde::{Serialize, Deserialize};
use ndarray::{Array2, Array1, Axis};
use rayon::prelude::*;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct Predictor {
    threshold: f32,
}

impl Predictor {
    pub fn new(threshold: f32) -> Self {
        Predictor { threshold }
    }

    pub fn predict_active_neurons(&self, preactivations: &Array1<f32>) -> Vec<bool> {
        preactivations.iter()
            .map(|&x| x > self.threshold)
            .collect()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Model {
    matrix: Array2<f32>,  // Preallocated FFN matrix (up + down project)
    pointers: Vec<usize>, // Original neuron indices
    bias: Array1<f32>,   // Bias for up project
    num_used: usize,     // Number of active rows
    last_k_active: Vec<usize>,  // Last k active neuron indices
    precision_config: Vec<PrecisionLevel>,
    predictor: Predictor,
    chunk_size: usize,   // 32KiB chunks
    d_model: usize,      // Model dimension
}

impl Model {
    pub fn new(size: usize, quantization_results: &[QuantizationResult]) -> Self {
        let d_model = 768;  // Example dimension (adjust based on model)
        let req_i = size / d_model;  // Max neurons from validation set
        let matrix = Array2::zeros((req_i, 2 * d_model));  // Preallocated matrix
        let pointers = vec![0; req_i];
        let bias = Array1::zeros(req_i);
        Model {
            matrix,
            pointers,
            bias,
            num_used: 0,
            last_k_active: vec![],
            precision_config: quantization_results.iter().map(|r| r.precision.clone()).collect(),
            predictor: Predictor::new(0.1),
            chunk_size: 32 * 1024,
            d_model,
        }
    }

    pub async fn load_from_flash(&mut self, file_path: &str) {
        let file = File::open(file_path).await.unwrap();
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0u8; self.chunk_size];

        let chunks: Vec<_> = (0..self.matrix.shape()[0])
            .step_by(self.chunk_size / (self.d_model * 4 * 2)) // 4 bytes per f32, 2 for up/down
            .collect();

        chunks.par_iter().for_each(|&start_row| {
            let mut local_buffer = buffer.clone();
            let mut local_reader = BufReader::new(File::open(file_path).unwrap());
            let start = start_row * self.d_model * 4 * 2;
            local_reader.read_exact(&mut local_buffer).unwrap();

            for (i, chunk) in local_buffer.chunks(4).enumerate() {
                let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                let row = start_row + i / (self.d_model * 2);
                let col = i % (self.d_model * 2);
                if row < self.matrix.shape()[0] {
                    self.matrix[[row, col]] = value;
                }
            }
        });
    }

    pub fn quantize(&mut self, quantization_results: &[QuantizationResult]) {
        self.precision_config = quantization_results.iter()
            .map(|r| r.precision.clone())
            .collect();
        for (i, precision) in self.precision_config.iter().enumerate() {
            match precision {
                PrecisionLevel::Bit4 => self.matrix.slice_mut(s![i, ..]).iter_mut().for_each(|x| *x *= 0.0625),
                PrecisionLevel::Bit8 => self.matrix.slice_mut(s![i, ..]).iter_mut().for_each(|x| *x *= 0.00390625),
                PrecisionLevel::Bit16 => {}
            }
        }
    }

    pub fn delete_neurons(&mut self, inactive_neurons: &[usize]) {
        let k = self.last_k_active.len().min(10); // Last 10 active neurons
        let mut new_last_k = self.last_k_active.clone();
        for &neuron in inactive_neurons {
            if let Some(idx) = self.pointers.iter().position(|&p| p == neuron) {
                if idx < self.num_used {
                    // Replace with most recent neuron
                    if let Some(last) = new_last_k.pop() {
                        self.matrix.slice_mut(s![idx, ..]).assign(&self.matrix.slice(s![self.num_used - 1, ..]));
                        self.pointers[idx] = self.pointers[self.num_used - 1];
                        self.bias[idx] = self.bias[self.num_used - 1];
                    }
                    self.num_used -= 1;
                }
            }
        }
        self.last_k_active = new_last_k;
    }

    pub fn add_neurons(&mut self, new_neurons: &[usize], weights: &[f32], biases: &[f32]) {
        let start = self.num_used;
        let end = (self.num_used + new_neurons.len()).min(self.matrix.shape()[0]);
        if end > start {
            self.matrix.slice_mut(s![start..end, ..]).assign(&Array2::from_shape_vec((end - start, self.matrix.shape()[1]), weights.to_vec()).unwrap());
            for (i, &neuron) in new_neurons.iter().enumerate().take(end - start) {
                self.pointers[start + i] = neuron;
                self.bias[start + i] = biases[i];
            }
            self.num_used = end;
        }
    }

    pub fn compute_ffn(&self, input: &Array1<f32>) -> Array1<f32> {
        let preactivations = input.dot(&self.matrix.slice(s![..self.num_used, ..self.d_model]));
        let active_neurons = self.predictor.predict_active_neurons(&preactivations);
        let mut output = Array1::zeros(preactivations.len());
        for (i, &active) in active_neurons.iter().enumerate() {
            if active {
                let up_proj = preactivations[i] + self.bias[i];
                let down_proj = self.matrix.slice(s![i, self.d_model..]).dot(&Array1::ones(self.d_model));
                output[i] = (up_proj * down_proj).max(0.0); // Combined up/down with ReLU
            }
        }
        output
    }
}