/*
Inference module for KVQuant-RS. This module handles the inference logic for the KVQuant model, including loading the model from flash storage, predicting active neurons, and managing the inference process.
Zeta Reticula is a Rust library for working with KVQuant models. It provides functionality to manage key-value caches, inference, and model interactions.
*/

use ndarray::{Array2, Array1, s};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use crate::quantizer::{QuantizationResult, PrecisionLevel};
use crate::kv_cache::KVQuantCache;
use crate::role_inference::RoleInferer;
use crate::mesolimbic::MesolimbicSystem;
use crate::KVQuantConfig;
use crate::tableaux::YoungTableau;
use crate::block::DataBlock;
use crate::role_inference::RoleInferenceResult;
use crate::mesolimbic::SalienceResult;
use crate::pb::kv_quant_service_server::KVQuantServiceServer;
use crate::pb::{KVQuantRequest, KVQuantResponse};
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use std::collections::HashMap;
use std::sync::RwLock;
use neon::prelude::*;
use crate::pb::KVQuantService;

#[derive(Serialize, Deserialize)]
pub struct KVQuantModel {
    pub matrix: Array2<f32>,  // Preallocated FFN matrix (up + down project)
    pub pointers: Vec<usize>, // Original neuron indices
    pub bias: Array1<f32>,   // Bias for up project
    pub num_used: usize,     // Number of active rows
    pub last_k_active: Vec<usize>,  // Last k active neuron indices
    pub precision_config: Vec<PrecisionLevel>,
    pub predictor: RoleInferer,
    pub chunk_size: usize,   // 32KiB chunks
    pub d_model: usize,      // Model dimension
}

impl KVQuantModel {
    pub fn new(size: usize, quantization_results: &[QuantizationResult]) -> Self {
        let d_model = 768;  // Example dimension (adjust based on model)
        let req_i = size / d_model;  // Max neurons from validation set
        let matrix = Array2::zeros((req_i, 2 * d_model));  // Preallocated matrix
        let pointers = vec![0; req_i];
        let bias = Array1::zeros(req_i);
        KVQuantModel {
            matrix,
            pointers,
            bias,
            num_used: 0,
            last_k_active: vec![],
            precision_config: quantization_results.iter().map(|r| r.precision.clone()).collect(),
            predictor: RoleInferer::new(0.1), // Example threshold
            chunk_size: 32 * 1024,
            d_model,
        }
    }

    pub async fn load_from_flash(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path).await?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).await?;
        
        let model: KVQuantModel = bincode::deserialize(&buffer)?;
        self.matrix = model.matrix;
        self.pointers = model.pointers;
        self.bias = model.bias;
        self.num_used = model.num_used;
        self.last_k_active = model.last_k_active;
        self.precision_config = model.precision_config;
        self.predictor = model.predictor;
        self.chunk_size = model.chunk_size;
        self.d_model = model.d_model;

        Ok(())
    }
}

pub async fn load_model_from_flash(file_path: &str) -> Result<KVQuantModel, Box<dyn std::error::Error>> {
    let file = File::open(file_path).await?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).await?;
    
    let model: KVQuantModel = bincode::deserialize(&buffer)?;
    Ok(model)
}

    pub async fn load_from_flash(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path).await?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).await?;
        
        let model: KVQuantModel = bincode::deserialize(&buffer)?;
        self.matrix = model.matrix;
        self.pointers = model.pointers;
        self.bias = model.bias;
        self.num_used = model.num_used;
        self.last_k_active = model.last_k_active;
        self.precision_config = model.precision_config;
        self.predictor = model.predictor;
        self.chunk_size = model.chunk_size;
        self.d_model = model.d_model;

        Ok(())
    }
pub async fn load_model_from_flash(file_path: &str) -> Result<KVQuantModel, Box<dyn std::error::Error>> {
    let file = File::open(file_path).await?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).await?;
    
    let model: KVQuantModel = bincode::deserialize(&buffer)?;
    Ok(model)
}