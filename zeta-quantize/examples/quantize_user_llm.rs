use zeta_quantize::{
    neurosymbolic_engine::{NeurosymbolicQuantizationEngine, UserLLMConfig},
    cli::PrecisionLevel,
    config::Config,
};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🚀 Zeta Reticula Neurosymbolic Quantization Engine Example");
    println!("=========================================================");

    // Load configuration
    let config = Config::default();
    
    // Initialize the neurosymbolic quantization engine
    println!("📦 Initializing neurosymbolic quantization engine...");
    let mut engine = NeurosymbolicQuantizationEngine::new(config).await?;

    // Configure user LLM for quantization
    let llm_config = UserLLMConfig {
        model_path: "examples/sample_model.safetensors".to_string(),
        model_type: "llama".to_string(),
        target_precision: PrecisionLevel::Int4,
        preserve_phonemes: true,
        use_federated_anns: true,
    };

    println!("🧠 User LLM Configuration:");
    println!("   Model Type: {}", llm_config.model_type);
    println!("   Target Precision: {:?}", llm_config.target_precision);
    println!("   Preserve Phonemes: {}", llm_config.preserve_phonemes);
    println!("   Use Federated ANNS: {}", llm_config.use_federated_anns);

    // Perform neurosymbolic quantization
    println!("\n⚡ Starting neurosymbolic quantization...");
    let output_path = Path::new("examples/quantized_model.safetensors");
    
    let result = engine.quantize_user_llm(llm_config, output_path).await?;

    // Display results
    println!("\n✅ Quantization completed successfully!");
    println!("📊 Results Summary:");
    println!("   Original Size: {} bytes", result.original_size);
    println!("   Quantized Size: {} bytes", result.quantized_size);
    println!("   Memory Reduction: {:.2}x", result.original_size as f64 / result.quantized_size as f64);
    println!("   Phoneme Preservation: {:.1}%", result.phoneme_preservation_score * 100.0);
    
    println!("\n🧮 KV Cache Statistics:");
    println!("   Prefill Tokens: {}", result.kv_cache_stats.prefill_tokens);
    println!("   Cache Hits: {}", result.kv_cache_stats.cache_hits);
    println!("   Cache Misses: {}", result.kv_cache_stats.cache_misses);
    println!("   Compression Ratio: {:.2}", result.kv_cache_stats.compression_ratio);

    println!("\n🎯 Precision Distribution:");
    for (precision, count) in result.precision_distribution {
        println!("   {:?}: {} tokens", precision, count);
    }

    println!("\n🔬 Neurosymbolic Analysis:");
    println!("   Salience Results: {} phoneme-aware tokens analyzed", result.salience_analysis.len());
    
    let homogeneity_preserved = result.salience_analysis.iter()
        .filter(|r| r.homogeneity_preserved)
        .count();
    println!("   Homogeneity Preserved: {}/{} tokens", homogeneity_preserved, result.salience_analysis.len());

    println!("\n🎉 Neurosymbolic quantization demonstration complete!");
    
    Ok(())
}
