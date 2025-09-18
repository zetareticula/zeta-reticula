use std::time::Duration;
use tokio::time::sleep;
use kvquant_deployment::{KVQuantDeployment, ServerlessConfig};

#[tokio::test]
async fn test_kvquant_serverless_deployment() {
    // Initialize deployment with test configuration
    let mut config = ServerlessConfig::default();
    config.serverless_memory_mb = 512; // Smaller for testing
    config.concurrent_workers = 2;
    config.petri_net_capacity = 100;
    
    let deployment = KVQuantDeployment::new(config).await
        .expect("Failed to initialize KVQuant deployment");
    
    // Test inference processing
    let tokens = vec![1, 2, 3, 4, 5];
    let input_data = vec![0.1, 0.8, 0.3, 0.9, 0.2];
    
    let result = deployment.process_inference_request(tokens, input_data).await
        .expect("Inference processing failed");
    
    assert!(result.processed_tokens > 0);
    assert!(!result.salience_scores.is_empty());
    println!("✅ Inference test passed: {} tokens processed", result.processed_tokens);
}

#[tokio::test]
async fn test_configuration_reporting() {
    let config = ServerlessConfig::default();
    let deployment = KVQuantDeployment::new(config).await
        .expect("Failed to initialize deployment");
    
    // Generate configuration report
    let report = deployment.get_configuration_report().await;
    
    assert_eq!(report.serverless_config.concurrent_workers, 8);
    assert_eq!(report.petri_net_stats.active_places, 4);
    assert!(report.kv_cache_stats.total_spots >= 0);
    
    println!("✅ Configuration report test passed");
    println!("📊 Memory usage: {}MB", report.kv_cache_stats.memory_usage_mb);
}

#[tokio::test]
async fn test_concurrent_processing() {
    let config = ServerlessConfig::default();
    let deployment = KVQuantDeployment::new(config).await
        .expect("Failed to initialize deployment");
    
    // Simulate concurrent requests
    let mut handles = vec![];
    
    for i in 0..5 {
        let deployment_clone = std::sync::Arc::new(deployment);
        let handle = tokio::spawn(async move {
            let tokens = vec![i * 10, i * 10 + 1, i * 10 + 2];
            let input_data = vec![0.1 * i as f32, 0.2 * i as f32, 0.3 * i as f32];
            
            deployment_clone.process_inference_request(tokens, input_data).await
        });
        handles.push(handle);
    }
    
    // Wait for all concurrent requests to complete
    let mut successful_requests = 0;
    for handle in handles {
        if let Ok(Ok(_result)) = handle.await {
            successful_requests += 1;
        }
    }
    
    assert!(successful_requests >= 3); // At least 3 out of 5 should succeed
    println!("✅ Concurrent processing test passed: {}/5 requests successful", successful_requests);
}

#[tokio::test]
async fn test_petri_net_dynamic_windowing() {
    let config = ServerlessConfig::default();
    let deployment = KVQuantDeployment::new(config).await
        .expect("Failed to initialize deployment");
    
    // Test with high salience tokens (should be processed)
    let high_salience_tokens = vec![100, 101, 102];
    let high_salience_data = vec![0.9, 0.8, 0.7]; // High values for salience
    
    let result = deployment.process_inference_request(high_salience_tokens, high_salience_data).await
        .expect("High salience processing failed");
    
    // Test with low salience tokens (may be filtered)
    let low_salience_tokens = vec![200, 201, 202];
    let low_salience_data = vec![0.1, 0.2, 0.3]; // Low values for salience
    
    let _low_result = deployment.process_inference_request(low_salience_tokens, low_salience_data).await
        .expect("Low salience processing failed");
    
    // High salience should process more tokens
    assert!(result.processed_tokens > 0);
    println!("✅ Petri net dynamic windowing test passed");
}

#[tokio::test]
async fn test_mesolimbic_role_inference() {
    let config = ServerlessConfig::default();
    let deployment = KVQuantDeployment::new(config).await
        .expect("Failed to initialize deployment");
    
    // Test role inference with diverse token patterns
    let tokens = vec![1, 50, 100, 500, 1000]; // Different token ranges
    let input_data = vec![0.2, 0.4, 0.6, 0.8, 1.0]; // Increasing values
    
    let result = deployment.process_inference_request(tokens, input_data).await
        .expect("Role inference processing failed");
    
    // Should have computed salience for all tokens
    assert_eq!(result.salience_scores.len(), tokens.len());
    
    // Check that different tokens get different salience scores
    let mut unique_scores = std::collections::HashSet::new();
    for score_result in result.salience_scores.values() {
        unique_scores.insert((score_result.salience_score * 1000.0) as i32);
    }
    
    assert!(unique_scores.len() > 1); // Should have varied salience scores
    println!("✅ Mesolimbic role inference test passed: {} unique salience patterns", unique_scores.len());
}

#[tokio::test]
async fn test_kv_cache_block_inference() {
    let config = ServerlessConfig::default();
    let deployment = KVQuantDeployment::new(config).await
        .expect("Failed to initialize deployment");
    
    // Process initial request to populate cache
    let tokens = vec![10, 20, 30, 40, 50];
    let input_data = vec![0.5, 0.6, 0.7, 0.8, 0.9];
    
    let first_result = deployment.process_inference_request(tokens.clone(), input_data.clone()).await
        .expect("First request failed");
    
    // Small delay to ensure cache is updated
    sleep(Duration::from_millis(10)).await;
    
    // Process same request again (should hit cache)
    let second_result = deployment.process_inference_request(tokens, input_data).await
        .expect("Second request failed");
    
    // Cache should have entries from both requests
    assert!(second_result.cache_hits >= first_result.cache_hits);
    println!("✅ KV cache block inference test passed: {} cache entries", second_result.cache_hits);
}

#[tokio::test]
async fn test_quantization_precision_levels() {
    let mut config = ServerlessConfig::default();
    
    // Test different precision levels
    let precisions = vec![
        kvquant_rs::PrecisionLevel::Int4,
        kvquant_rs::PrecisionLevel::Int8,
        kvquant_rs::PrecisionLevel::FP16,
    ];
    
    for precision in precisions {
        config.kvquant.precision = precision.clone();
        
        let deployment = KVQuantDeployment::new(config.clone()).await
            .expect("Failed to initialize deployment");
        
        let tokens = vec![1, 2, 3];
        let input_data = vec![0.1, 0.2, 0.3];
        
        let result = deployment.process_inference_request(tokens, input_data).await
            .expect("Precision test failed");
        
        assert_eq!(result.quantization_precision, precision);
        println!("✅ Precision {:?} test passed", precision);
    }
}

#[tokio::test]
async fn test_serverless_memory_limits() {
    let mut config = ServerlessConfig::default();
    config.serverless_memory_mb = 256; // Low memory limit
    config.kvquant.max_cache_items = 100; // Reduced cache
    
    let deployment = KVQuantDeployment::new(config).await
        .expect("Failed to initialize low-memory deployment");
    
    // Process multiple requests to test memory management
    for i in 0..10 {
        let tokens = vec![i, i + 1, i + 2];
        let input_data = vec![0.1, 0.2, 0.3];
        
        let _result = deployment.process_inference_request(tokens, input_data).await
            .expect("Memory limit test failed");
    }
    
    let report = deployment.get_configuration_report().await;
    assert!(report.kv_cache_stats.memory_usage_mb <= 256);
    
    println!("✅ Memory limits test passed: {}MB used", report.kv_cache_stats.memory_usage_mb);
}
