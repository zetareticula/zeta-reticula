use crate::cli::PrecisionLevel;
use crate::error::Result;
use crate::quantization::ErrorMetrics;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub precision: PrecisionLevel,
    pub memory_factor: f64,
    pub duration_secs: f64,
    pub error_metrics: ErrorMetrics,
}

/// Format bytes in human-readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}

/// Save benchmark results to JSON file
pub async fn save_benchmark_results(results: &[BenchmarkResult], path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    fs::write(path, json).await?;
    Ok(())
}

/// Load benchmark results from JSON file
pub async fn load_benchmark_results(path: &Path) -> Result<Vec<BenchmarkResult>> {
    let content = fs::read_to_string(path).await?;
    let results: Vec<BenchmarkResult> = serde_json::from_str(&content)?;
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
}
