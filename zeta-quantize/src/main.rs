use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, warn};

mod cli;
mod config;
mod engine;
mod error;
mod memory;
mod model;
mod quantization;
mod tokenizer;
mod utils;
mod neurosymbolic_engine;

use cli::{Args, Commands};
use config::Config;
use engine::QuantizationEngine;
use neurosymbolic_engine::{NeurosymbolicQuantizationEngine, UserLLMConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    let args = Args::parse();
    
    info!("Starting Zeta Reticula Quantization Engine v{}", env!("CARGO_PKG_VERSION"));
    
    match args.command {
        Commands::Quantize {
            input_path,
            output_path,
            precision,
            config_path,
            batch_size,
            memory_limit,
            validate_memory,
        } => {
            let config = if let Some(config_path) = config_path {
                Config::from_file(&config_path)?
            } else {
                Config::default()
            };

            let mut engine = QuantizationEngine::new(config)?;
            
            if validate_memory {
                info!("Memory validation enabled - performing algebraic memory assertions");
            }

            let result = engine.quantize_model(
                &input_path,
                &output_path,
                precision,
                batch_size,
                memory_limit,
                validate_memory,
            ).await?;

            info!("Quantization completed successfully");
            info!("Memory savings: {:.2}x reduction", result.memory_reduction_factor);
            info!("Model size: {} -> {}", 
                utils::format_bytes(result.original_size),
                utils::format_bytes(result.quantized_size)
            );
        }
        Commands::Benchmark {
            model_path,
            precision_levels,
            output_path,
        } => {
            let config = Config::default();
            let engine = QuantizationEngine::new(config)?;
            
            let results = engine.benchmark_quantization(
                &model_path,
                &precision_levels,
            ).await?;

            if let Some(output_path) = output_path {
                utils::save_benchmark_results(&results, &output_path)?;
            }

            for result in results {
                info!("Precision: {:?}, Memory Factor: {:.2}x, Time: {:.2}s",
                    result.precision, result.memory_factor, result.duration_secs);
            }
        }
        Commands::Validate { model_path } => {
            let config = Config::default();
            let engine = QuantizationEngine::new(config)?;
            
            let validation = engine.validate_model(&model_path).await?;
            
            if validation.is_valid {
                info!("Model validation passed");
            } else {
                warn!("Model validation failed: {}", validation.error_message.unwrap_or_default());
            }
        }
        Commands::QuantizeUserLLM {
            model_path,
            output_path,
            model_type,
            precision,
            preserve_phonemes,
            use_federated_anns,
            config_path,
        } => {
            let config = if let Some(config_path) = config_path {
                Config::from_file(&config_path)?
            } else {
                Config::default()
            };

            info!("Initializing neurosymbolic quantization engine");
            let mut engine = NeurosymbolicQuantizationEngine::new(config).await?;

            let llm_config = UserLLMConfig {
                model_path: model_path.to_string_lossy().to_string(),
                model_type,
                target_precision: precision,
                preserve_phonemes,
                use_federated_anns,
            };

            info!("Starting neurosymbolic quantization for user LLM");
            let result = engine.quantize_user_llm(llm_config, &output_path).await?;

            info!("Neurosymbolic quantization completed successfully");
            info!("Original size: {}", utils::format_bytes(result.original_size));
            info!("Quantized size: {}", utils::format_bytes(result.quantized_size));
            info!("Memory reduction: {:.2}x", result.original_size as f64 / result.quantized_size as f64);
            info!("Phoneme preservation score: {:.2}%", result.phoneme_preservation_score * 100.0);
            info!("KV cache prefill tokens: {}", result.kv_cache_stats.prefill_tokens);
            info!("KV cache compression ratio: {:.2}", result.kv_cache_stats.compression_ratio);
            
            // Display precision distribution
            info!("Precision distribution:");
            for (precision, count) in result.precision_distribution {
                info!("  {:?}: {} tokens", precision, count);
            }
        }
    }

    Ok(())
}
