use thiserror::Error;

/// Unified error type for the input layer
#[derive(Error, Debug)]
pub enum UnifiedInputError {
    #[error("Tokenizer error: {0}")]
    Tokenizer(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Model loading error: {0}")]
    ModelLoading(String),

    #[error("Safetensors error: {0}")]
    Safetensors(#[from] safetensors::SafeTensorError),

    #[error("HuggingFace Hub error: {0}")]
    HuggingFaceHub(String),

    #[error("Candle error: {0}")]
    Candle(#[from] candle_core::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid model format: {0}")]
    InvalidFormat(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Tokenization failed: {0}")]
    TokenizationFailed(String),

    #[error("Embedding generation failed: {0}")]
    EmbeddingFailed(String),
}

/// Result type alias for unified input operations
pub type Result<T> = std::result::Result<T, UnifiedInputError>;

impl From<tokenizers::Error> for UnifiedInputError {
    fn from(err: tokenizers::Error) -> Self {
        UnifiedInputError::Tokenizer(err.to_string())
    }
}

impl From<hf_hub::api::sync::ApiError> for UnifiedInputError {
    fn from(err: hf_hub::api::sync::ApiError) -> Self {
        UnifiedInputError::HuggingFaceHub(err.to_string())
    }
}
