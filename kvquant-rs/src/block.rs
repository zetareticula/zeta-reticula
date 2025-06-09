use serde::{Serialize, Deserialize};
use std::collections::HashMap;

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
    pub data: HashMap<u32, f32>,  // token_id -> value
    pub pointers: Vec<usize>,     // Neuron indices
    pub biases: Vec<f32>,         // Bias values
}

impl DataBlock {
    pub fn new(id: usize) -> Self {
        DataBlock {
            id,
            state: BlockState::Free,
            data: HashMap::new(),
            pointers: vec![],
            biases: vec![],
        }
    }

    pub fn write(&mut self, token_id: u32, value: f32, pointer: usize, bias: f32) {
        if self.state == BlockState::Free {
            self.data.insert(token_id, value);
            self.pointers.push(pointer);
            self.biases.push(bias);
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
        self.state = BlockState::Free;
    }
}