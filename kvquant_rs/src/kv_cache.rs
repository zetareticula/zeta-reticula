// Lightweight module shim to expose KV cache types under `kv_cache`.
// The concrete implementations live in `block.rs`.

pub use crate::block::{
    LogStructuredKVCache,
    KVCache,
    initialize_kv_cache,
};
