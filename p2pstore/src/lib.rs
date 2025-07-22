

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KVCache {
    pub buffers: Vec<AllocatedBufferDescriptor>,
    pub positional_encoding: Option<Vec<i32>>, // Decoupled for truncation
}

impl KVCache {
    pub fn new(buffers: Vec<AllocatedBufferDescriptor>) -> Self {
        KVCache {
            buffers,
            positional_encoding: None,
        }
    }
}

pub trait TransferEngine {
    async fn async_load(&self, cache: &KVCache, hbm: &mut Vec<KVCache>) -> Result<(), TransferEngineError>;
    async fn async_save(&self, cache: Vec<KVCache>) -> Result<(), TransferEngineError>;
    // ... (other methods)
}

#[derive(Debug)]
pub enum TransferEngineError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Buffer overflow: {0}")]
    Overflow(String),
}