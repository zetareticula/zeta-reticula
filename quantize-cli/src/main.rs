// Copyright 2025 ZETA RETICULA
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

//! Zeta Reticula Quantize CLI
//! 
//! A powerful CLI tool for quantizing, optimizing, and running inference on LLMs
//! with neurosymbolic salience analysis and routing.

mod app;
mod cli;
mod error;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use app::{QuantizeApp, AppConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Initialize application with config
    let config = AppConfig {
        verbose: cli.verbose,
        format: cli.format,
    };
    
    let mut app = QuantizeApp::new(config)?;
    
    // Initialize required systems
    app.init_salience_system().await?;
    app.init_ns_router().await?;
    app.init_quantizer().await?;
    
    // Execute the requested command
    match cli.command {
        cli::Commands::Quantize(args) => {
            app.quantize_model(
                &args.input,
                &args.output,
                args.bits,
                args.use_salience,
                args.update,
                args.enable_time_direction,
                args.forward_time,
                args.time_context_scale,
            ).await?;
        }
        cli::Commands::Infer(args) => {
            let result = app.run_inference(
                &args.model,
                &args.input,
                args.use_router,
                args.max_tokens,
                args.enable_time_direction,
                args.forward_time,
                args.time_context_scale,
            ).await?;
            
            // Print the inference result
            println!("{}", result);
        }
        cli::Commands::Optimize(args) => {
            app.optimize_model(
                &args.model,
                &args.output,
                args.use_kv_cache,
            ).await?;
        }
        cli::Commands::Convert(args) => {
            app.convert_model(
                &args.input,
                &args.output,
                &args.format,
            ).await?;
        }
    }
    
    Ok(())
}
 