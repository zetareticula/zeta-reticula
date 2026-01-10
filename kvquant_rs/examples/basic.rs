//! Basic example of using the kvquant_rs library

use kvquant_rs::{
    KVQuantizer,
    KVQuantConfig,
    PrecisionLevel,
};

fn main() {
    println!("Starting KVQuant example");
    
    // Create a new KVQuantizer with default configuration
    let config = KVQuantConfig::default();
    let kvq = KVQuantizer::new(config.clone());
    
    // Example input data
    let input = vec![0.0, 0.5, -0.5, 1.0, -1.0, 0.25, -0.75];
    
    // Perform quantization
    match kvq.quantize(&input) {
        Ok(quantized) => {
            println!("Quantization successful!");
            println!("Input size: {} floats", input.len());
            println!("Quantized size: {} bytes", quantized.len());
            
            // Show precision level
            match config.precision {
                PrecisionLevel::Int8 => println!("Using Int8 precision"),
                PrecisionLevel::Int4 => println!("Using Int4 precision"),
                PrecisionLevel::Int2 => println!("Using Int2 precision"),
                PrecisionLevel::Bit1 => println!("Using Bit1 precision"),
            }
            
            // Dequantize to verify
            match kvq.dequantize(&quantized) {
                Ok(dequantized) => {
                    println!("\nDequantization successful!");
                    println!("Original vs Dequantized:");
                    for (i, (orig, deq)) in input.iter().zip(dequantized.iter()).enumerate() {
                        let error = (orig - deq).abs();
                        println!("  [{:2}] {:.4} -> {:.4} (error: {:.4})", i, orig, deq, error);
                    }
                }
                Err(e) => println!("Dequantization failed: {}", e),
            }
        }
        Err(e) => println!("Quantization failed: {}", e),
    }
}
