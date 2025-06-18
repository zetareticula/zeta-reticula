use clap::Parser;
use serde::{Serialize, Deserialize};
use std::default::Default;
use std::fmt;
use std::error::Error;
use log::LevelFilter;
use log::info;
use env_logger::Builder;
use ns_router_rs::initialize_ns_router;
use salience_engine::quantizer::{SalienceQuantizer, TokenFeatures};
use salience_engine::quantizer::{QuantizationResult, PrecisionLevel};
use std::fs;
use std::io::{self, BufWriter};
use std::path::PathBuf;
use std::io::Write;
use serde_json;
use csv::Writer;
use ns_router_rs::KVCacheConfig;
use ns_router_rs::NSRoutingPlan;
use llm_rs::InferenceOutput;
use std::fs::OpenOptions;
use quantize_cli::CliConfig;
use std::error::Error;
use std::io::Error as IoError;

/// Zeta Reticula Quantize CLI
/// This CLI quantizes LLMs using neurosymbolic salience, reading input text, quantizing tokens, routing inference requests, and outputting results in JSON or CSV format.
/// It supports verbose logging and allows for domain-specific salience through a theory key.
/// It integrates with the SalienceQuantizer for token quantization and NSRouter for routing inference requests.
/// The output includes quantization results, routing plans, inference outputs, and performance metrics.
pub mod quantize_cli;


//// Zeta Reticula Quantize CLI Configuration
#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author, version, about = "Zeta Reticula Quantize CLI: Quantize LLMs with neurosymbolic salience", long_about = None)]
pub struct CliConfig {
    #[arg(long, default_value = "input.txt", help = "Input file containing text to quantize")]
    pub input_file: String,

    #[arg(long, default_value = "output.json", help = "Output file for quantization results")]
    pub output_file: String,

    #[arg(long, default_value = "json", help = "Output format: json or csv")]
    pub format: String,

    #[arg(long, default_value = "default", help = "Theory key for domain-specific salience (e.g., 'legal')")]
    pub theory_key: String,

    #[arg(long, default_value = "user123", help = "User ID for routing")]
    pub user_id: String,

    #[arg(long, help = "Enable verbose logging")]
    pub verbose: bool,
}

impl CliConfig {
    pub fn parse_args() -> Self {
        CliConfig::parse()
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_csv(&self) -> Result<String, csv::Error> {
        let mut wtr = Writer::from_writer(vec![]);
        wtr.serialize(self)?;
        let data = String::from_utf8(wtr.into_inner()?)?;
        Ok(data)
    }

    pub fn write_output(&self, results: &str) -> io::Result<()> {
        let path = PathBuf::from(&self.output_file);
        let mut file = BufWriter::new(fs::OpenOptions::new().write(true).create(true).truncate(true).open(path)?);
        if self.format == "json" {
            file.write_all(results.as_bytes())?;
        } else if self.format == "csv" {
            let mut wtr = Writer::from_writer(file);
            wtr.write_record(results.split(','))?;
            wtr.flush()?;
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unsupported output format"));
        }
        Ok(())
    }

    pub fn read_input(&self) -> io::Result<String> {
        fs::read_to_string(&self.input_file).map_err(|e| io::Error::new(io::ErrorKind::NotFound, e))
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        CliConfig {
            input_file: "input.txt".to_string(),
            output_file: "output.json".to_string(),
            format: "json".to_string(),
            theory_key: "default".to_string(),
            user_id: "user123".to_string(),
            verbose: false,
        }
    }

}

impl fmt::Display for CliConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CliConfig {{ input_file: {}, output_file: {}, format: {}, theory_key: {}, user_id: {}, verbose: {} }}",
               self.input_file, self.output_file, self.format, self.theory_key, self.user_id, self.verbose)
    }
}




/// Initializes the logger based on the verbosity setting
/// 
/// # Arguments:
/// /// * `verbose` - If true, sets the log level to Info; otherwise, sets it to Warn
 pub fn init_logger(verbose: bool) {
    Builder::new()
        .filter_level(if verbose { LevelFilter::Info } else { LevelFilter::Warn })
        .init();
    info!("Logger initialized with verbosity: {}", if verbose { "Info" } else { "Warn" });
}

/// Main entry point for the Zeta Reticula Quantize CLI
/// /// # Returns:
/// /// * `Result<(), Box<dyn Error>>` - Ok if successful, Err if there was an error

pub fn run_cli() -> Result<(), Box<dyn Error>> {
    let config = CliConfig::parse_args();
    init_logger(config.verbose);
    
    info!("Starting Zeta Reticula Quantize CLI with config: {}", config);

    let input = std::fs::read_to_string(&config.input_file)?;
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
    let routing_plan = router.route_inference(&input, &config.user_id)?;

    // Output results
    info!("Quantization results: {:?}", quantization_results);
    info!("Routing plan: {:?}", routing_plan);

    Ok(())
}

/// Error handling for CLI operations
/// /// # Arguments:
/// /// * `msg` - The error message to display
/// /// # Returns:
/// /// * `Box<dyn Error>` - A boxed error containing the error message
pub fn handle_error(msg: &str) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::Other, msg))

}

/// Custom error type for CLI operations
pub struct CliError {
    message: String,
}

impl CliError {
    pub fn new(message: &str) -> Self {
        CliError {
            message: message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CliConfig::default();
        assert_eq!(config.input_file, "input.txt");
        assert_eq!(config.output_file, "output.json");
        assert_eq!(config.format, "json");
        assert_eq!(config.theory_key, "default");
        assert_eq!(config.user_id, "user123");
        assert!(!config.verbose);
    }

    #[test]
    fn test_parse_args() {
        let args = vec![
            "quantize-cli",
            "--input-file", "test_input.txt",
            "--output-file", "test_output.json",
            "--format", "csv",
            "--theory-key", "test_theory",
            "--user-id", "test_user",
            "--verbose"
        ];
        let config = CliConfig::parse_from(args);
        assert_eq!(config.input_file, "test_input.txt");
        assert_eq!(config.output_file, "test_output.json");
        assert_eq!(config.format, "csv");
        assert_eq!(config.theory_key, "test_theory");
        assert_eq!(config.user_id, "test_user");
        assert!(config.verbose);
    }
}