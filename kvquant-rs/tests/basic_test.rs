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
    let kvq = KVQuantizer::new(config);
    
    // Simple test to verify the quantizer was created
    // We're just testing that we can create it without panicking
    assert!(true);
}

#[test]
fn test_quantization() {
    // Create a default configuration
    let config = KVQuantConfig::default();
    let kvq = KVQuantizer::new(config);
    
    // Test data
    let token_id = 42;
    let value = 0.5;
    let pointer = 0;
    let bias = 0.1;
    let vector_id = 1;
    let graph_entry = (0, vec![1, 2, 3]);
    
    // Perform quantization
    let result = kvq.quantize(token_id, value, pointer, bias, vector_id, graph_entry);
    
    // For now, just verify that we got some result
    // The actual quantization logic would have more specific assertions
    assert!(result.is_some());
}
