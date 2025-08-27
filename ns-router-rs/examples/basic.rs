// Copyright 2025 ZETA RETICULA INC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


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
    
    
    let router = NSRouter::new();
    
   
    let context_analyzer = NSContextAnalyzer::new();
    

    let token_features = vec![]; // Add actual token features here
    
  
    let plan = router.get_routing_plan("example_prompt", &token_features).await?;
    
    info!("Routing plan: {:?}", plan);
    

    info!("Selected model: {}", plan.model_config.model);
    info!("Execution strategy: {}", plan.execution_strategy);
    
    Ok(())
}
