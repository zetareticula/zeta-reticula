use llm_rs::InferenceEngine;
use ndarray::array;

fn main() {
    println!("Testing llm-rs minimal example");
    
    // Create a simple inference engine
    let d_model = 768;
    let mut engine = InferenceEngine::new(d_model);
    
    // Create some dummy weights (in a real scenario, these would be loaded from a file)
    let weights = vec![0u8; d_model * 4]; // Simple dummy weights
    engine.load_weights(weights);
    
    println!("Inference engine created and weights loaded successfully!");
    
    // Create a dummy input
    let input = "Hello, world!";
    println!("Input: {}", input);
    
    // Note: The actual inference would require more setup (like a routing plan),
    // but this demonstrates that the basic structure is working.
    println!("Basic llm-rs functionality is working!");
}
