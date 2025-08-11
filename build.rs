use std::env;
use std::process::Command;
use std::path::Path;

fn main() {
    // Print build information
    println!("cargo:rerun-if-changed=build.rs");
    
    // Get the current directory
    let current_dir = env::current_dir().expect("Failed to get current directory");
    
    // Check if we're in a workspace context
    let in_workspace = current_dir.ends_with("workspace");
    
    // Set the working directory
    let workspace_dir = if in_workspace {
        current_dir.clone()
    } else {
        current_dir.join("workspace")
    };
    
    // Get the target crate from environment variable or use default
    let target_crate = env::var("BUILD_CRATE").unwrap_or_else(|_| "".to_string());
    
    // Build command
    let mut cmd = Command::new("cargo");
    
    // Set the working directory
    cmd.current_dir(&workspace_dir);
    
    // Add build arguments
    cmd.arg("build");
    
    // Add release flag if specified
    if env::var("PROFILE").map(|p| p == "release").unwrap_or(false) {
        cmd.arg("--release");
    }
    
    // Add target crate if specified
    if !target_crate.is_empty() {
        cmd.arg("-p").arg(target_crate);
    }
    
    // Execute the build
    let status = cmd.status().expect("Failed to execute cargo build");
    
    if !status.success() {
        panic!("Build failed with status: {}", status);
    }
}
