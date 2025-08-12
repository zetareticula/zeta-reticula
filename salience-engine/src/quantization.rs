use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantizationResult {
    pub row: usize,
    pub col: usize,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrecisionLevel {
    Bit4,
    Bit8,
    Bit16,
    Bit32,
}

impl Default for PrecisionLevel {
    fn default() -> Self {
        Self::Bit32
    }
}
