//! Basic example of using the NS Router

use ns_router_rs::{
    NSRouter, 
    NSContextAnalyzer, 
    ModelConfig, 
    KVCacheConfig,
    NSRoutingPlan,
    PrecisionLevel
};
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();
    
    info!("Initializing NS Router...");
    
    // Create a new router
    let router = NSRouter::new();
    
    // Create a context analyzer
    let context_analyzer = NSContextAnalyzer::new();
    
    // Example token features (in a real application, these would come from your tokenizer)
    let token_features = vec![]; // Add actual token features here
    
    // Get a routing plan
    let plan = router.get_routing_plan("example_prompt", &token_features).await?;
    
    info!("Routing plan: {:?}", plan);
    
    // Example of using the routing plan
    info!("Selected model: {}", plan.model_config.model);
    info!("Execution strategy: {}", plan.execution_strategy);
    
    Ok(())
}
