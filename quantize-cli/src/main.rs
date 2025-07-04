mod config;
pub mod output;

use std::fs;
use log::{info, LevelFilter};
use env_logger::Builder;
use salience_engine::quantizer::{SalienceQuantizer, TokenFeatures};
use ns_router_rs::initialize_ns_router;
use tokio::runtime::Runtime;
use ns_router_rs::KVCacheConfig;
use ns_router_rs::NSRoutingPlan;
use llm_rs::InferenceOutput;
use serde::{Serialize, Deserialize};
use serde_json;
use csv::Writer;
use std::io::Write;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::{self, BufWriter};
use std::error::Error;
use clap::Parser;
use std::fmt;
use std::default::Default;
use config::CliConfig;
use salience_engine::quantizer::{QuantizationResult, PrecisionLevel};



// Zeta Reticula Quantize CLI
// Main entry point for the Zeta Reticula Quantize CLI, which quantizes LLMs using neurosymbolic salience.
//// This CLI reads input text, quantizes tokens based on salience, routes inference requests,
//// and outputs the results in a specified format (JSON or CSV).
/// // The CLI supports verbose logging and allows for domain-specific salience through a theory key.
/// // It integrates with the SalienceQuantizer for token quantization and NSRouter for routing inference requests.
/// // The output includes quantization results, routing plans, inference outputs, and performance metrics.



/// # Zeta Reticula Quantize CLI
/// This CLI tool is designed to quantize tokens for large language models (LLMs) using neurosymbolic salience.
/// It reads input text from a file, quantizes the tokens based on their salience,
/// routes inference requests using a neurosymbolic router,
/// and writes the output to a specified file in JSON or CSV format.





/// Main function for the Zeta Reticula Quantize CLI
/// This function initializes the logger, reads input from a file,
/// quantizes tokens using the SalienceQuantizer,
/// routes inference requests using the NSRouter,
/// and writes the output to a specified file.
/// # Arguments:
/// * `config` - The CLI configuration containing input file, output file, theory key, user ID, and verbosity flag.
/// # Returns:
/// * `Result<(), std::io::Error>` - Returns Ok if successful, or an error if any operation fails.

fn main() -> std::io::Result<()> {
    let config = config::CliConfig::parse_args();

    Builder::new()
        .filter_level(if config.verbose { LevelFilter::Info } else { LevelFilter::Warn })
        .init();

    info!("Starting Zeta Reticula Quantize CLI with config: {:?}", config);

    let input = fs::read_to_string(&config.input_file)?;
    info!("Read input: {} tokens", input.split_whitespace().count());

    let quantizer = SalienceQuantizer::new(0.7);
    let router = initialize_ns_router();

    let token_features: Vec<TokenFeatures> = input.split_whitespace()
        .enumerate()
        .map(|(idx, _)| TokenFeatures {
            token_id: idx as u32,
            frequency: 0.5,
            sentiment_score: 0.0,
            context_relevance: 0.5,
            role: "".to_string(),
        })
        .collect();

    info!("Quantizing tokens with theory key: {}", config.theory_key);
    let (quantization_results, _tableau) = quantizer.quantize_tokens(token_features, &config.theory_key);

    info!("Routing inference for user: {}", config.user_id);
    let rt = Runtime::new()?;
    let (routing_plan, inference_output) = rt.block_on(async {
        router.route_inference(&input, &config.user_id).await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    })?;

    let data_size_mb = (input.split_whitespace().count() as f32 * 32.0 * 1024.0) / (1024.0 * 1024.0);
    let throughput_mb_per_sec = data_size_mb / (inference_output.latency_ms / 1000.0);

    let active_tokens = quantization_results.iter()
        .filter(|r| matches!(r.precision, PrecisionLevel::Bit16))
        .count();
    let sparsity_ratio = 1.0 - (active_tokens as f32 / quantization_results.len() as f32);

    let num_used = quantization_results.len();
    let last_k_active = routing_plan.kv_cache_config.priority_tokens.iter().map(|&t| t as usize).collect();

    // Mock ANNS metrics
    let anns_recall = 0.95; // Example recall
    let anns_throughput = 1000.0 / (inference_output.latency_ms / 1000.0); // Queries per second

    let output = output::CliOutput {
        quantization_results,
        routing_plan,
        inference_output,
        input_tokens: input.split_whitespace().count(),
        throughput_mb_per_sec,
        sparsity_ratio,
        num_used,
        last_k_active,
        anns_recall,
        anns_throughput,
    };
    output::write_output(&output, &config)?;

    info!("Quantization complete. Output written to {}", config.output_file);
    Ok(())
}

#[derive(Parser, Debug)]
#[clap(name = "quantize-cli", version = "1.0", author = "Zeta Reticula")]
pub struct CliConfig {
    #[clap(long, default_value = "input.txt")]
    pub input_file: PathBuf,

    #[clap(long, default_value = "output.json")]
    pub output_file: PathBuf,

    #[clap(long, default_value = "json")]
    pub format: String,

    #[clap(long, default_value = "default")]
    pub theory_key: String,

    #[clap(long, default_value = "user123")]
    pub user_id: String,

    #[clap(short, long)]
    pub verbose: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        CliConfig {
            input_file: PathBuf::from("input.txt"),
            output_file: PathBuf::from("output.json"),
            format: "json".to_string(),
            theory_key: "default".to_string(),
            user_id: "user123".to_string(),
            verbose: false,
        }
    }
}

impl fmt::Display for CliConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Input File: {}\nOutput File: {}\nFormat: {}\nTheory Key: {}\nUser ID: {}\nVerbose: {}",
               self.input_file.display(), self.output_file.display(), self.format, self.theory_key, self.user_id, self.verbose)
    }
}

// Output module for writing results to file
mod output {
    use super::*;
    use serde_json::to_string;
    use std::fs::File;
    use std::io::{self, BufWriter};

    #[derive(Serialize, Deserialize)]
    pub struct CliOutput {
        pub quantization_results: Vec<QuantizationResult>,
        pub routing_plan: NSRoutingPlan,
        pub inference_output: InferenceOutput,
        pub input_tokens: usize,
        pub throughput_mb_per_sec: f32,
        pub sparsity_ratio: f32,
        pub num_used: usize,
        pub last_k_active: Vec<usize>,
        pub anns_recall: f32,
        pub anns_throughput: f32,
    }

    pub fn write_output(output: &CliOutput, config: &CliConfig) -> io::Result<()> {
        let file = File::create(&config.output_file)?;
        let mut writer = BufWriter::new(file);

        if config.format == "json" {
            let json_output = to_string(output)?;
            writer.write_all(json_output.as_bytes())?;
        } else if config.format == "csv" {
            let mut csv_writer = Writer::from_writer(writer);
            for result in &output.quantization_results {
                csv_writer.serialize(result)?;
            }
            csv_writer.flush()?;
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unsupported format"));
        }

        Ok(())
    }
}