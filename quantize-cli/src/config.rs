use clap::Parser;
use serde::{Serialize, Deserialize};

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