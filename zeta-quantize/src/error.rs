use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuantizationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Model loading error: {0}")]
    ModelLoad(String),

    #[error("Tensor operation error: {0}")]
    TensorOp(String),

    #[error("Memory allocation error: {0}")]
    Memory(String),

    #[error("Quantization failed: {0}")]
    Quantization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Memory assertion failed: expected {expected}, got {actual}")]
    MemoryAssertion { expected: u64, actual: u64 },

    #[error("Precision not supported: {0:?}")]
    UnsupportedPrecision(crate::cli::PrecisionLevel),

    #[error("Candle error: {0}")]
    Candle(#[from] candle_core::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, QuantizationError>;

impl QuantizationError {
    pub fn model_load<S: Into<String>>(msg: S) -> Self {
        Self::ModelLoad(msg.into())
    }

    pub fn tensor_op<S: Into<String>>(msg: S) -> Self {
        Self::TensorOp(msg.into())
    }

    pub fn memory<S: Into<String>>(msg: S) -> Self {
        Self::Memory(msg.into())
    }

    pub fn quantization<S: Into<String>>(msg: S) -> Self {
        Self::Quantization(msg.into())
    }

    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    pub fn unsupported_format<S: Into<String>>(msg: S) -> Self {
        Self::UnsupportedFormat(msg.into())
    }

    pub fn memory_assertion(expected: u64, actual: u64) -> Self {
        Self::MemoryAssertion { expected, actual }
    }
}
