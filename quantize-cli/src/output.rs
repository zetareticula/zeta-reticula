use serde::Serialize;
use std::fs::File;
use std::io::Write;
use csv::Writer;
use salience_engine::quantizer::QuantizationResult;
use llm_rs::InferenceOutput;
use ns_router_rs::KVCacheConfig;

#[derive(Serialize)]
pub struct CliOutput {
    pub quantization_results: Vec<QuantizationResult>,
    pub routing_plan: ns_router_rs::NSRoutingPlan,
    pub inference_output: InferenceOutput,
    pub input_tokens: usize,
    pub throughput_mb_per_sec: f32,
    pub sparsity_ratio: f32,
    pub num_used: usize,
    pub last_k_active: Vec<usize>,
    pub anns_recall: f32,  // Mock recall metric
    pub anns_throughput: f32,  // Throughput for ANNS queries
}

pub fn write_output(output: &CliOutput, config: &super::config::CliConfig) -> std::io::Result<()> {
    match config.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(output)?;
            let mut file = File::create(&config.output_file)?;
            file.write_all(json.as_bytes())?;
        }
        "csv" => {
            let mut wtr = Writer::from_path(&config.output_file)?;
            wtr.write_record(&["token_id", "precision", "salience_score", "row", "role", "role_confidence"])?;
            for result in &output.quantization_results {
                wtr.write_record(&[
                    result.token_id.to_string(),
                    format!("{:?}", result.precision),
                    result.salience_score.to_string(),
                    result.row.to_string(),
                    result.role.clone(),
                    result.role_confidence.to_string(),
                ])?;
            }
            wtr.flush()?;
        }
        _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unsupported format")),
    }
    Ok(())
}