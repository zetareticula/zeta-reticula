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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the output directory for generated code
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    
    // Path to the proto file relative to the crate root
    let proto_file = "../zeta-sidecar/proto/sidecar.proto";
    
    // Compile the proto file
    tonic_build::configure()
        .build_server(true)
        .out_dir(&out_dir)
        .compile(&[proto_file], &[".."])?;
    
    // Re-run the build script if the proto file changes
    println!("cargo:rerun-if-changed={}", proto_file);
    
    Ok(())
}
