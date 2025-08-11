#!/bin/bash
set -e

# Change to the project root
cd "$(dirname "$0")"

# Function to build a crate with specific features
build_crate() {
    local crate=$1
    echo "Building $crate..."
    
    # Backup the Cargo.toml
    cp "$crate/Cargo.toml" "$crate/Cargo.toml.bak"
    
    # Comment out problematic dependencies
    if [ "$crate" == "llm-rs" ]; then
        sed -i.bak 's/^salience-engine =/# salience-engine =/' "$crate/Cargo.toml"
    fi
    
    if [ "$crate" == "ns-router-rs" ]; then
        sed -i.bak 's/^salience-engine =/# salience-engine =/' "$crate/Cargo.toml"
    fi
    
    if [ "$crate" == "salience-engine" ]; then
        sed -i.bak 's/^llm-rs =/# llm-rs =/' "$crate/Cargo.toml"
    fi
    
    # Build the crate with the correct package name (replace hyphens with underscores)
    local pkg_name=$(basename "$crate" | tr '-' '_')
    cargo build -p $pkg_name
    
    # Restore the original Cargo.toml
    mv "$crate/Cargo.toml.bak" "$crate/Cargo.toml"
}

# Build in the correct order
for crate in shared kvquant-rs ns-router-rs llm-rs salience-engine quantize-cli api; do
    if [ -d "$crate" ]; then
        build_crate "$crate"
    else
        echo "Skipping $crate (not found)"
    fi
done

echo "All crates built successfully!"
