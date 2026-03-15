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

// Lightweight module shim to expose KV cache types under `kv_cache`.
// The concrete implementations live in `block.rs`.

use serde::{Serialize, Deserialize};


pub use crate::block::{
    LogStructuredKVCache,
    KVCache,
    initialize_kv_cache,
    DataBlock,
    BlockState,
    KVQuantizer,
};

pub use crate::{
    KVQuantConfig,
    PrecisionLevel,
    QuantizationResult,
    QuantizationData,
    RoleInferer,
    RoleInferenceResult,
    MesolimbicSystem,
    SalienceResult,
};

// High-level, experimental API for state-of-the-art KV quantization backends.
// This does NOT change existing behavior; it is an opt‑in extension surface.
//
// The concrete, hardware‑specific / algorithmic backends should live in their
// own modules (e.g., `awq`, `gptq`, `llm_int8`, `llm_int4_kv`, etc.) and
// implement this trait. This keeps `kv_cache` a lightweight shim while still
// exposing a modern abstraction.

/// Fine‑grained quantization scheme for KV tensors.
///
/// The classic `PrecisionLevel` is coarse; modern systems combine low‑bit
/// storage with smarter scaling / grouping. This enum captures that while
/// remaining backend‑agnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KVQuantScheme {
    /// Tensor‑wise symmetric int8 (good baseline, fast on most hardware).
    Int8Symmetric,
    /// Per‑channel / per‑head int8 (better accuracy, still simple).
    Int8PerChannel,
    /// Per‑group int4 with learned scales (GPTQ / AWQ‑style for KV).
    Int4Groupwise,
    /// Mixed precision: keys in 8‑bit, values in 4‑bit (common for KV).
    MixedK8V4Groupwise,
    /// 3‑bit groupwise (emerging hardware, experimental).
    Int3Groupwise,
    /// 2‑bit ultra‑low precision with aggressive outlier handling.
    Int2OutlierAware,
    /// 1‑bit binary KV with separate high‑precision outlier buffer.
    BinaryWithOutliers,
}

/// Describes the layout of the KV tensor to be quantized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KVLayout {
    pub num_layers: u16,
    pub num_heads: u16,
    /// Head dimension (d_k).
    pub head_dim: u16,
    /// Whether K and V are interleaved or stored separately.
    pub interleaved_kv: bool,
}

/// A generic, state‑of‑the‑art KV quantization backend.
///
/// This trait is intentionally minimal and async‑friendly so that implementations
/// can offload to GPU, custom accelerators, or SIMD kernels.
pub trait KVQuantBackend: Send + Sync {
    /// Quantize a full KV "snapshot" (e.g., a block or a layer range) into
    /// backend‑specific compressed form.
    ///
    /// `kv` is expected to be shaped logically as:
    ///   [num_layers, num_heads, seq_len, head_dim]
    /// in row‑major (contiguous) layout in memory.
    ///
    /// Implementations are free to reorder / pack internally.
    fn quantize_kv_block(
        &self,
        kv: &[f32],
        layout: KVLayout,
        scheme: KVQuantScheme,
    ) -> QuantizationResult<Vec<u8>>;

    /// Dequantize a previously quantized KV block back into f32.
    ///
    /// This is needed for validation, debugging, or backends that may choose
    /// to fall back to full precision for some operations.
    fn dequantize_kv_block(
        &self,
        packed: &[u8],
        layout: KVLayout,
        scheme: KVQuantScheme,
    ) -> QuantizationResult<Vec<f32>>;

    /// Optionally re‑quantize in place when changing scheme (e.g., dynamic
    /// "precision scheduling" based on salience / attention patterns).
    fn requantize_kv_block(
        &self,
        packed: &[u8],
        layout: KVLayout,
        from: KVQuantScheme,
        to: KVQuantScheme,
    ) -> QuantizationResult<Vec<u8>>;

    /// Inform the backend about new salience statistics so it can adapt:
    ///   - lower precision for low‑salience regions
    ///   - keep high precision for highly attended heads / layers
    fn update_salience_profile(
        &self,
        salience: &[(u32, f32)], // (token_id, salience_score)
    );
}

/// A thin, pluggable wrapper that connects `KVCache` / `LogStructuredKVCache`
/// with a chosen `KVQuantBackend`. This is purely an interface shim; the
/// actual wiring happens in consuming crates.
#[derive(Clone)]
pub struct KVQuantEngine<B: KVQuantBackend> {
    pub backend: B,
    pub layout: KVLayout,
    pub scheme: KVQuantScheme,
}

impl<B: KVQuantBackend> KVQuantEngine<B> {
    pub fn new(backend: B, layout: KVLayout, scheme: KVQuantScheme) -> Self {
        Self { backend, layout, scheme }
    }

    /// Quantize a raw KV tensor with the configured layout and scheme.
    pub fn quantize(&self, kv: &[f32]) -> QuantizationResult<Vec<u8>> {
        self.backend.quantize_kv_block(kv, self.layout, self.scheme)
    }

    /// Dequantize a packed KV tensor.
    pub fn dequantize(&self, packed: &[u8]) -> QuantizationResult<Vec<f32>> {
        self.backend
            .dequantize_kv_block(packed, self.layout, self.scheme)
    }

    /// Switch to a new scheme (e.g., when adapting precision on the fly)
    /// and return the re‑quantized representation.
    pub fn switch_scheme(
        &mut self,
        packed: &[u8],
        new_scheme: KVQuantScheme,
    ) -> QuantizationResult<Vec<u8>> {
        let out = self.backend.requantize_kv_block(
            packed,
            self.layout,
            self.scheme,
            new_scheme,
        )?;
        self.scheme = new_scheme;
        Ok(out)
    }

    /// Surface salience updates to the backend so it can adapt internal
    /// thresholds, outlier handling, or mixed‑precision policies.
    pub fn propagate_salience(&self, salience_scores: &[(u32, f32)]) {
        self.backend.update_salience_profile(salience_scores);
    }
}

