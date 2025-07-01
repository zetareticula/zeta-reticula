use serde::{Serialize, Deserialize};

/// Represents the result of quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationResult {
    /// The original value before quantization
    pub original: f32,
    /// The quantized value
    pub quantized: f32,
    /// The scale factor used for quantization
    pub scale: f32,
    /// The zero point used for quantization (for asymmetric quantization)
    pub zero_point: Option<i32>,
    /// The precision level used for quantization
    pub precision: PrecisionLevel,
}

/// Represents different precision levels for quantization
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PrecisionLevel {
    /// 4-bit quantization
    Bit4,
    /// 8-bit quantization
    Bit8,
    /// 16-bit quantization
    Bit16,
    /// 32-bit quantization (full precision)
    Bit32,
    /// Custom bit width
    Custom(u8),
}

impl Default for PrecisionLevel {
    fn default() -> Self {
        PrecisionLevel::Bit32
    }
}

impl QuantizationResult {
    /// Creates a new QuantizationResult
    pub fn new(
        original: f32,
        quantized: f32,
        scale: f32,
        zero_point: Option<i32>,
        precision: PrecisionLevel,
    ) -> Self {
        Self {
            original,
            quantized,
            scale,
            zero_point,
            precision,
        }
    }

    /// Calculates the quantization error
    pub fn error(&self) -> f32 {
        (self.original - self.quantized).abs()
    }

    /// Calculates the relative error
    pub fn relative_error(&self) -> f32 {
        if self.original == 0.0 {
            0.0
        } else {
            self.error() / self.original.abs()
        }
    }
}
