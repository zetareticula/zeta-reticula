//! Basic example of using the kvquant_rs library

use kvquant_rs::{
    KVQuantizer,
    KVQuantConfig,
    PrecisionLevel,
    QuantizationResult,
};
use ndarray::Array1;
use log::info;

fn main() {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting KVQuant example");
    
    // Create a new KVQuantizer with default configuration
    let config = KVQuantConfig::default();
    let kvq = KVQuantizer::new(config);
    
    // Example token ID and value
    let token_id = 42;
    let value = 0.5;
    let pointer = 0;
    let bias = 0.1;
    let vector_id = 1;
    let graph_entry = (0, vec![1, 2, 3]);
    
    // Perform quantization
    if let Some(result) = kvq.quantize(token_id, value, pointer, bias, vector_id, graph_entry) {
        info!("Quantization result: {:?}", result);
        
        // Example of using the result
        match result.precision {
            PrecisionLevel::Bit8 => {
                println!("Using 8-bit precision");
            }
            PrecisionLevel::Bit16 => {
                println!("Using 16-bit precision");
            }
            PrecisionLevel::Bit32 => {
                println!("Using 32-bit precision");
            }
            PrecisionLevel::Medium => {
                println!("Using medium precision");
            }
        }
        
        println!("Salience score: {}", result.salience_score);
        println!("Row: {}", result.row);
        println!("Role: {}", result.role);
        println!("Role confidence: {}", result.role_confidence);
    } else {
        println!("Quantization failed");
    }
}
