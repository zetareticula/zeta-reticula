

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
}

/// Arguments for the infer command
#[derive(Parser, Debug)]
pub struct InferArgs {
    /// Model path
    #[clap(short, long)]
    pub model: PathBuf,
    
    /// Input text or path to input file
    #[clap(short, long)]
    pub input: String,
    
    /// Use neurosymbolic routing
    #[clap(long)]
    pub use_ns_router: bool,
    
    /// Maximum tokens to generate
    #[clap(short, long, default_value = "100")]
    pub max_tokens: usize,
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
