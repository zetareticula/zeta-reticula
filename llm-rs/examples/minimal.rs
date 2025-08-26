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
