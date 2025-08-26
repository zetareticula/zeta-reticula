fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("src/proto")
        .compile(
            &["proto/master.proto"],
            &["proto/"], // specify the root location to search proto dependencies
        )?;
    Ok(())
}
