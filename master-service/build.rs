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
    let proto_file = "proto/master.proto";
    let out_dir = "src/proto";
    
    // Create the output directory if it doesn't exist
    std::fs::create_dir_all(out_dir)?;
    
    tonic_build::configure()
        .out_dir(out_dir)
        .compile(
            &[proto_file],
            &["."], // specify the root location to search proto dependencies
        )?;
        
    // Re-run the build script if the proto file changes
    println!("cargo:rerun-if-changed={}", proto_file);
    Ok(())
}
