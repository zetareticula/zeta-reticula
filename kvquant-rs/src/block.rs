use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::vec::Vec;
use std::sync::Arc;
use dashmap::DashMap;
use std::sync::Mutex;
use std::sync::RwLock;
use dashmap::mapref::entry::Entry;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicI32;
use crate::tableaux::YoungTableau;
use crate::role_inference::{RoleInferer, RoleInferenceResult};
use crate::mesolimbic::{MesolimbicSystem, SalienceResult};
use crate::role_inference::RoleTheory;
use crate::quantizer::{QuantizationResult, PrecisionLevel};
use crate::quantizer::KVQuantConfig;

#[derive(Serialize, Deserialize)]
pub struct KVQuantizer {
    pub config: KVQuantConfig,
    pub data_blocks: DashMap<usize, DataBlock>, // Concurrent access to data blocks
    pub role_inferer: Arc<RoleInferer>,
    pub mesolimbic_system: Arc<MesolimbicSystem>,
}

impl KVQuantizer {
    pub fn new(config: KVQuantConfig) -> Self {
        let role_inferer = Arc::new(RoleInferer::new(10, 5)); // 10 outer, 5 inner iterations
        let mesolimbic_system = Arc::new(MesolimbicSystem::new());
        KVQuantizer {
            config,
            data_blocks: DashMap::new(),
            role_inferer,
            mesolimbic_system,
        }
    }

    pub fn quantize(&self, token_id: u32, value: f32, pointer: usize, bias: f32, vector_id: u32, graph_entry: (usize, Vec<usize>)) -> Option<QuantizationResult> {
        let block_id = (token_id as usize) % self.config.block_size;
        let mut block = self.data_blocks.entry(block_id).or_insert_with(|| DataBlock::new(block_id));

        if block.state == BlockState::Free || block.state == BlockState::Valid {
            block.write(token_id, value, pointer, bias, vector_id, graph_entry);
            Some(QuantizationResult {
                token_id,
                precision: self.config.precision_level,
                salience_score: value * self.config.salience_threshold,
                row: block.id,
                role: "default".to_string(), // Placeholder for role
                role_confidence: 1.0, // Placeholder for confidence
            })
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct KVQuantConfig {
    pub block_size: usize,
    pub spot_capacity: usize,
    pub salience_threshold: f32,
    pub precision_level: PrecisionLevel,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum PrecisionLevel {
    Bit16,
    Bit32,
    Bit64,
}



#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum BlockState {
    Free,
    Valid,
    Obsolete,
    Invalid,
}

#[derive(Serialize, Deserialize)]
pub struct DataBlock {
    pub id: usize,
    pub state: BlockState,
    pub data: HashMap<u32, f32>,
    pub pointers: Vec<usize>,
    pub biases: Vec<f32>,
    pub vector_ids: Vec<u32>,  // ANNS vector-IDs
    pub navigation_graph: HashMap<usize, Vec<usize>>,  // Navigation graph entries
}

impl DataBlock {
    pub fn new(id: usize) -> Self {
        DataBlock {
            id,
            state: BlockState::Free,
            data: HashMap::new(),
            pointers: vec![],
            biases: vec![],
            vector_ids: vec![],
            navigation_graph: HashMap::new(),
        }
    }

    pub fn write(&mut self, token_id: u32, value: f32, pointer: usize, bias: f32, vector_id: u32, graph_entry: (usize, Vec<usize>)) {
        if self.state == BlockState::Free {
            self.data.insert(token_id, value);
            self.pointers.push(pointer);
            self.biases.push(bias);
            self.vector_ids.push(vector_id);
            self.navigation_graph.insert(graph_entry.0, graph_entry.1);
            self.state = BlockState::Valid;
        }
    }

    pub fn unmap(&mut self) {
        if self.state == BlockState::Valid {
            self.state = BlockState::Obsolete;
        }
    }

    pub fn invalidate(&mut self) {
        if self.state == BlockState::Obsolete {
            self.state = BlockState::Invalid;
        }
    }

    pub fn erase(&mut self) {
        self.data.clear();
        self.pointers.clear();
        self.biases.clear();
        self.vector_ids.clear();
        self.navigation_graph.clear();
        self.state = BlockState::Free;
    }
}