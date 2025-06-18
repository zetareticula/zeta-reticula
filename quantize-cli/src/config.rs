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