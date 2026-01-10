//! Basic test for KVQuant functionality

use kvquant_rs::{
    KVQuantizer,
    KVQuantConfig,
    PrecisionLevel,
};

#[test]
fn test_kvquantizer_creation() {
    // Create a default configuration
    let config = KVQuantConfig::default();
    
    // Create a new KVQuantizer
    let _kvq = KVQuantizer::new(config);
    
    // Simple test to verify the quantizer was created
    // We're just testing that we can create it without panicking
    assert!(true);
}

#[test]
fn test_quantization() {
    // Create a default configuration
    let config = KVQuantConfig::default();
    let kvq = KVQuantizer::new(config);
    
    // Test data - using the actual quantize API which takes a slice of f32
    let input = vec![0.0, 0.5, -0.5, 1.0, -1.0];
    
    // Perform quantization
    let result = kvq.quantize(&input);
    
    // Verify quantization succeeded
    assert!(result.is_ok());
    
    // Verify we can dequantize
    let quantized = result.unwrap();
    let dequantized = kvq.dequantize(&quantized);
    assert!(dequantized.is_ok());
}
