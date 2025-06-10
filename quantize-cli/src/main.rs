use std::fs;
use log::{info, LevelFilter};
use env_logger::Builder;
use salience_engine::quantizer::{SalienceQuantizer, TokenFeatures};
use ns_router_rs::initialize_ns_router;
use tokio::runtime::Runtime;

mod config;
mod output;

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