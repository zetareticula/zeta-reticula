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
