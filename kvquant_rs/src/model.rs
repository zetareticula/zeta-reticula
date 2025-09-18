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


/*
Zeta Reticula is a Rust library for working with KVQuant models. It provides functionality to manage key-value caches, inference, and model interactions.
*/

use ndarray::{Array2, Array1};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::marker::PhantomData;
use crate::kvquant_config::{KVQuantConfig, PrecisionLevel, QuantizationResult, QuantizationData, QuantizationDataTrait};
use crate::role_inferer::RoleInferer;
use crate::mesolimbic_system::MesolimbicSystem;
use crate::block::{KVQuantizer, KVCache};
// Re-export the client for use in other modules
pub use crate::pb::sidecar_service_client::SidecarServiceClient;

#[derive(Serialize, Deserialize)]
pub struct KVQuantModel<T: QuantizationDataTrait> {
    pub matrix: Array2<f32>,  // Preallocated FFN matrix (up + down project)
    pub pointers: Vec<usize>, // Original neuron indices
    pub bias: Array1<f32>,   // Bias for up project
    pub num_used: usize,     // Number of active rows
    pub last_k_active: Vec<usize>,  // Last k active neuron indices
    pub precision_config: Vec<PrecisionLevel>,
    pub predictor: RoleInferer,
    pub chunk_size: usize,   // 32KiB chunks
    pub d_model: usize,      // Model dimension
    _phantom: PhantomData<T>,
}

impl<T: QuantizationDataTrait> KVQuantModel<T> {
    pub fn new(size: usize, quantization_results: &[QuantizationResult<T>]) -> Self {
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
            precision_config: quantization_results.iter().map(|r| r.as_ref().map(|data| data.precision()).unwrap_or(PrecisionLevel::Int8)).collect(),
            predictor: RoleInferer::with_iterations(1.0, 10, 5), // threshold: 1.0, 10 outer, 5 inner iterations
            chunk_size: 32 * 1024,
            d_model,
            _phantom: PhantomData,
        }
    }

    pub fn load_from_flash(&mut self, file_path: &str) -> io::Result<()> {
        // Backward-compatible wrapper that constructs ephemeral quantizer and cache
        let default_cfg = KVQuantConfig::default();
        let kvq = KVQuantizer::new(default_cfg.clone());
        let cache = KVCache::new(default_cfg);
        self.load_from_flash_with(file_path, &kvq, &cache)
    }

    /// Load serialized model data from a flash file in streaming fashion and materialize it
    /// via the KVQuantizer and KVCache. This method supports two wire formats:
    /// 1) bincode of Vec<SerializableEntry>
    /// 2) NDJSON fallback: one JSON object per line matching SerializableEntry
    pub fn load_from_flash_with(&mut self, file_path: &str, kvq: &KVQuantizer, cache: &KVCache) -> io::Result<()> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; self.chunk_size];
        let mut remainder: Vec<u8> = Vec::new();

        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                // flush remainder if any
                if !remainder.is_empty() {
                    match Self::process_chunk(&remainder, kvq, cache, self) {
                        Ok(()) => {
                            remainder.clear();
                        }
                        Err(ProcessError::NeedMoreData(_unconsumed)) => {
                            // End of file: ignore leftover partial entry
                        }
                        Err(ProcessError::Io(e)) => return Err(e),
                    }
                }
                break;
            }

            // Concatenate remainder + new bytes
            let mut chunk = Vec::with_capacity(remainder.len() + n);
            chunk.extend_from_slice(&remainder);
            chunk.extend_from_slice(&buffer[..n]);

            // Try to process; if partial bincode buffer remains, keep it in remainder
            match Self::process_chunk(&chunk, kvq, cache, self) {
                Ok(()) => {
                    remainder.clear();
                }
                Err(ProcessError::NeedMoreData(unconsumed)) => {
                    remainder = unconsumed;
                }
                Err(ProcessError::Io(e)) => return Err(e),
            }
        }
        Ok(())
    }
}

/// Minimal serializable entry for streaming ingestion through the quantizer and cache.
/// This captures the essentials needed by `KVQuantizer::quantize` and `KVCache::update`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableEntry {
    token_id: u32,
    value: f32,
    pointer: usize,
    bias: f32,
    vector_id: u32,
    /// Optional navigation adjacency list entry (key + neighbors)
    #[serde(default)]
    nav_key: Option<usize>,
    #[serde(default)]
    nav_neighbors: Option<Vec<usize>>,
    /// Optional precomputed salience; if absent, derive a simple heuristic.
    #[serde(default)]
    salience: Option<f32>,
}

enum ProcessError {
    NeedMoreData(Vec<u8>),
    Io(io::Error),
}

impl From<io::Error> for ProcessError {
    fn from(e: io::Error) -> Self { ProcessError::Io(e) }
}

impl<T: QuantizationDataTrait> KVQuantModel<T> {
    /// Attempt to parse and process a chunk.
    /// Strategy:
    /// - First attempt: bincode a Vec<SerializableEntry>. If deserialize fails due to EOF,
    ///   return NeedMoreData with original bytes.
    /// - Second attempt: NDJSON lines (each line a SerializableEntry JSON). Any partially read
    ///   last line is kept as remainder.
    fn process_chunk(chunk: &[u8], kvq: &KVQuantizer, cache: &KVCache, model: &mut Self) -> Result<(), ProcessError> {
        // Try bincode path
        if let Ok(entries) = bincode::deserialize::<Vec<SerializableEntry>>(chunk) {
            for e in entries { Self::ingest_entry(e, kvq, cache, model); }
            return Ok(());
        }

        // NDJSON fallback: split by lines; keep last incomplete line as remainder
        let mut last_newline = None;
        for (i, b) in chunk.iter().enumerate() {
            if *b == b'\n' { last_newline = Some(i); }
        }
        if let Some(end) = last_newline {
            let (complete, remainder) = chunk.split_at(end + 1);
            for line in complete.split(|c| *c == b'\n') {
                if line.is_empty() { continue; }
                if let Ok(entry) = serde_json::from_slice::<SerializableEntry>(line) {
                    Self::ingest_entry(entry, kvq, cache, model);
                }
            }
            return Err(ProcessError::NeedMoreData(remainder.to_vec()));
        }

        // No newline found and bincode failed: likely need more data
        Err(ProcessError::NeedMoreData(chunk.to_vec()))
    }

    fn ingest_entry(entry: SerializableEntry, kvq: &KVQuantizer, cache: &KVCache, model: &mut Self) {
        // Derive salience if not provided
        let salience = entry.salience.unwrap_or_else(|| entry.value.abs());
        let graph_entry = (entry.nav_key.unwrap_or(0usize), entry.nav_neighbors.unwrap_or_default());

        // Quantize writes into a block; result provides metadata (ignored here but could be used)
        let _qres: Option<QuantizationResult<QuantizationData>> = kvq.quantize(
            entry.token_id,
            entry.value,
            entry.pointer,
            entry.bias,
            entry.vector_id,
            graph_entry,
        );

        // Update KV cache with salience-aware write
        cache.update(entry.token_id, entry.value, salience, entry.pointer, entry.bias);

        // Update model book-keeping (pointers/biases/matrix layout)
        // Keep this lightweight: bump counters and last_k_active ring.
        model.num_used = model.num_used.saturating_add(1);
        if model.last_k_active.len() >= 1024 { model.last_k_active.remove(0); }
        model.last_k_active.push(entry.token_id as usize);
    }
}

    // pub fn predict_active_neurons(&self, preactivations: &Array1<f32>) -> Vec<bool> {
    //     self.predictor.predict_active_neurons(preactivations)
    // }

// KVQuantService is the gRPC service for KVQuant operations
#[derive(Default)]
pub struct KVQuantService {
    pub config: Option<HashMap<String, String>>,
    // pub kv_cache: Arc<RwLock<KVQuantCache>>, // Removed: KVQuantCache not defined
    pub role_inferer: Arc<RoleInferer>,
    pub mesolimbic_system: Arc<MesolimbicSystem>,
}

