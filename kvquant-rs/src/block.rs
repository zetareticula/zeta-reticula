use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::vec::Vec;
use std::sync::Arc;
use dashmap::DashMap;
use std::sync::Mutex;
use std::sync::RwLock;
use dashmap::mapref::entry::Entry;
use rand::Rng;

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