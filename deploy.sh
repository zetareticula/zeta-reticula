#!/bin/bash

# Exit on error
set -e

# Clean build artifacts
echo "Cleaning build artifacts..."
rm -rf target dist build *.wasm

# Build Rust components
echo "Building Rust components..."
cd api
cargo build --release --target wasm32-unknown-unknown --features wasm
# Build Python bindings
echo "Building Python bindings..."
python3 -m pip install maturin --upgrade
maturin build --release --strip --interpreter python3

# Build TypeScript/Next.js components
echo "Building TypeScript components..."
# Ensure node_modules is clean
rm -rf node_modules
npm ci
npm run build

# Prepare for deployment
echo "Preparing for deployment..."
# Copy necessary files
mkdir -p dist
# Copy WASM files
cp target/wasm32-unknown-unknown/release/*.wasm dist/
# Copy Python wheel
find dist -name "*.whl" -exec cp {} dist/ \;

# Deploy to Vercel
echo "Deploying to Vercel..."
vercel --prod --build-env NODE_ENV=production --build-env RUST_LOG=warn

echo "Deployment complete!"
