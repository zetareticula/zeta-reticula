use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use attention_store::store::AttentionStore;
use p2pstore::{KVCache, TransferEngine};
use client::Client;
use master_service::MasterService;

mod test_config;
use test_config::TestConfig;

#[tokio::test]
async fn test_attention_store_vault_integration() {
    // Initialize test configuration
    let test_config = TestConfig::new().await;
    
    // Initialize AttentionStore with test dependencies
    let transfer_engine = Arc::new(TransferEngine::new());
    let client = Arc::new(Client::new());
    let master_service = Arc::new(MasterService::new());
    
    let attention_store = AttentionStore::new(
        test_config.sync_manager.clone(),
        transfer_engine,
        client,
        master_service,
    ).expect("Failed to create AttentionStore");

    // 4. Test basic operations
    let session_id = "test-session".to_string();
    let test_tokens = vec![101, 102, 103]; // Example token IDs
    
    // Test prefill operation
    let result = attention_store.prefill(session_id.clone(), test_tokens.clone()).await;
    assert!(result.is_ok(), "Prefill operation failed");
    
    // Test decode operation
    let (next_token, _) = attention_store.decode(session_id.clone(), 101, vec![]).await.expect("Decode failed");
    assert_eq!(next_token, 102, "Unexpected next token");
    
    // Test cache eviction
    attention_store.evict().await;
    
    // Verify cache state after eviction
    // (Add assertions based on your eviction policy)
    
    // Cleanup
    attention_store.shutdown().await.expect("Failed to shutdown AttentionStore");
    test_config.sync_manager.shutdown().await.expect("Failed to shutdown SyncManager");
}

// Helper function to create a test KVCache
fn create_test_kv_cache() -> KVCache {
    KVCache {
        key: vec![0u8; 32],
        value: vec![0u8; 64],
        timestamp: chrono::Utc::now().timestamp() as u64,
        ttl: Some(3600),
    }
}
