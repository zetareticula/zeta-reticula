fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the output directory for generated code
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    
    // Compile the protobuf files
    tonic_build::configure()
        .build_server(true)
        .out_dir(out_dir)
        .compile(
            &["../zeta-sidecar/proto/sidecar.proto"],
            &[".."], // Location to search for imports
        )?;
    
    // Re-run the build script if the proto file changes
    println!("cargo:rerun-if-changed=../zeta-sidecar/proto/sidecar.proto");
    
    Ok(())
}
