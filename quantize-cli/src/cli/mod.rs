// Copyright 2025 ZETA RETICULA
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

//! Command-line interface for the Zeta Reticula quantization tool

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[cfg(test)]
mod tests;

/// Main CLI structure
#[derive(Parser, Debug)]
#[clap(name = "quantize-cli", version, about = "Zeta Reticula Model Quantization Tool")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
    
    /// Enable verbose output
    #[clap(short, long, global = true)]
    pub verbose: bool,
    
    /// Output format (json, yaml, toml)
    #[clap(short, long, default_value = "json", global = true)]
    pub format: String,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Quantize a model
    Quantize(QuantizeArgs),
    
    /// Run inference with quantization
    Infer(InferArgs),
    
    /// Optimize model with salience analysis
    Optimize(OptimizeArgs),
    
    /// Convert between model formats
    Convert(ConvertArgs),
}

/// Arguments for the quantize command
#[derive(Parser, Debug)]
pub struct QuantizeArgs {
    /// Input model path
    #[clap(short, long)]
    pub input: PathBuf,
    
    /// Output directory
    #[clap(short, long, default_value = "./output")]
    pub output: PathBuf,
    
    /// Quantization bits (4, 8, 16, 32)
    #[clap(short, long, default_value = "8")]
    pub bits: u8,
    
    /// Use salience-aware quantization
    #[clap(long)]
    pub use_salience: bool,
    
    /// Update existing quantized model with time directionality optimizations
    #[clap(long)]
    pub update: bool,
    
    /// Enable time directionality for quantization (requires --update)
    #[clap(long, requires = "update")]
    pub enable_time_direction: bool,
    
    /// Default time direction (true = forward, false = backward)
    #[clap(long, default_value = "true", requires = "enable_time_direction")]
    pub forward_time: bool,
    
    /// Time direction context scale factor
    #[clap(long, default_value = "1.0", requires = "enable_time_direction")]
    pub time_context_scale: f32,
}

/// Arguments for the infer command
#[derive(Parser, Debug)]
pub struct InferArgs {
    /// Path to the model file
    #[clap(short, long)]
    pub model: PathBuf,
    
    /// Input text for inference
    #[clap(short, long)]
    pub input: String,
    
    /// Use neuro-symbolic routing
    #[clap(short = 'r', long)]
    pub use_router: bool,
    
    /// Maximum number of tokens to generate
    #[clap(short = 'n', long, default_value = "128")]
    pub max_tokens: usize,
    
    /// Enable time directionality (forward/backward)
    #[clap(long)]
    pub enable_time_direction: bool,
    
    /// Default time direction (true = forward, false = backward)
    #[clap(long, default_value = "true")]
    pub forward_time: bool,
    
    /// Time direction context scale factor
    #[clap(long, default_value = "1.0")]
    pub time_context_scale: f32,
}

/// Arguments for the optimize command
#[derive(Parser, Debug)]
pub struct OptimizeArgs {
    /// Model path
    #[clap(short, long)]
    pub model: PathBuf,
    
    /// Output directory
    #[clap(short, long, default_value = "./optimized")]
    pub output: PathBuf,
    
    /// Use KV cache optimization
    #[clap(long)]
    pub use_kv_cache: bool,
}

/// Arguments for the convert command
#[derive(Parser, Debug)]
pub struct ConvertArgs {
    /// Input model path
    #[clap(short, long)]
    pub input: PathBuf,
    
    /// Output path
    #[clap(short, long)]
    pub output: PathBuf,
    
    /// Target format (gguf, safetensors, etc.)
    #[clap(short, long, default_value = "gguf")]
    pub format: String,
}
