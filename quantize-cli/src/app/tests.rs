use super::*;
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn test_app_initialization() {
    let config = AppConfig {
        verbose: true,
        format: "json".to_string(),
    };
    
    let app = QuantizeApp::new(config).unwrap();
    assert!(app.config.verbose);
    assert_eq!(app.config.format, "json");
}

#[tokio::test]
async fn test_salience_system_initialization() {
    let config = AppConfig::default();
    let mut app = QuantizeApp::new(config).unwrap();
    
    app.init_salience_system().await.unwrap();
    assert!(app.salience_system.is_some());
}

#[tokio::test]
async fn test_ns_router_initialization() {
    let config = AppConfig::default();
    let mut app = QuantizeApp::new(config).unwrap();
    
    app.init_ns_router().await.unwrap();
    assert!(app.ns_router.is_some());
}

#[tokio::test]
async fn test_quantizer_initialization() {
    let config = AppConfig::default();
    let mut app = QuantizeApp::new(config).unwrap();
    
    app.init_quantizer().await.unwrap();
    assert!(app.quantizer.is_some());
}

#[tokio::test]
async fn test_quantize_model() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("test_model.gguf");
    let output_path = temp_dir.path().join("quantized_model.gguf");
    
    // Create a dummy input file
    let mut file = File::create(&input_path).await.unwrap();
    file.write_all(b"dummy model data").await.unwrap();
    
    let config = AppConfig::default();
    let app = QuantizeApp::new(config).unwrap();
    
    // Test basic quantization
    let result = app.quantize_model(&input_path, &output_path, 8, false).await;
    assert!(result.is_ok());
    
    // Test with salience
    let result = app.quantize_model(&input_path, &output_path, 4, true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_inference() {
    let temp_dir = tempdir().unwrap();
    let model_path = temp_dir.path().join("test_model.gguf");
    
    // Create a dummy model file
    let mut file = File::create(&model_path).await.unwrap();
    file.write_all(b"dummy model data").await.unwrap();
    
    let config = AppConfig::default();
    let app = QuantizeApp::new(config).unwrap();
    
    // Test basic inference
    let result = app.run_inference(&model_path, "Test input", false, 50).await;
    assert!(result.is_ok());
    
    // Test with NS router
    let result = app.run_inference(&model_path, "Test input", true, 100).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_optimize_model() {
    let temp_dir = tempdir().unwrap();
    let model_path = temp_dir.path().join("test_model.gguf");
    let output_path = temp_dir.path().join("optimized_model.gguf");
    
    // Create a dummy model file
    let mut file = File::create(&model_path).await.unwrap();
    file.write_all(b"dummy model data").await.unwrap();
    
    let config = AppConfig::default();
    let app = QuantizeApp::new(config).unwrap();
    
    // Test optimization without KV cache
    let result = app.optimize_model(&model_path, &output_path, false).await;
    assert!(result.is_ok());
    
    // Test optimization with KV cache
    let result = app.optimize_model(&model_path, &output_path, true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_convert_model() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input_model.bin");
    let output_path = temp_dir.path().join("output_model.gguf");
    
    // Create a dummy input file
    let mut file = File::create(&input_path).await.unwrap();
    file.write_all(b"dummy model data").await.unwrap();
    
    let config = AppConfig::default();
    let app = QuantizeApp::new(config).unwrap();
    
    // Test conversion to GGUF
    let result = app.convert_model(&input_path, &output_path, "gguf").await;
    assert!(result.is_ok());
    
    // Test conversion to safetensors
    let output_path = temp_dir.path().join("output_model.safetensors");
    let result = app.convert_model(&input_path, &output_path, "safetensors").await;
    assert!(result.is_ok());
}
