# Zeta Reticula Build System

This document describes the modular build system for Zeta Reticula, designed for efficient compilation of independent components.

## Key Features

- **Modular Architecture**: Each component can be built independently
- **Incremental Compilation**: Faster rebuilds during development
- **Parallel Builds**: Utilizes all available CPU cores
- **Workspace Support**: Manages dependencies between crates

## Quick Start

1. **Build all components**:
   ```bash
   ./scripts/build.sh
   ```

2. **Build specific components**:
   ```bash
   ./scripts/build.sh agentflow-rs quantize-cli
   ```

3. **Customize build settings**:
   ```bash
   BUILD_MODE=debug FEATURES="cuda,opencl" ./scripts/build.sh
   ```

## Build Configuration

Edit `scripts/build.conf` to customize build settings:

```ini
# Build mode (debug or release)
BUILD_MODE=release

# Features to enable
FEATURES=default

# Number of parallel jobs (0 = auto-detect)
JOBS=0
```

## Environment Variables

- `BUILD_MODE`: Set to `debug` or `release`
- `TARGET`: Target platform (e.g., `x86_64-apple-darwin`)
- `FEATURES`: Comma-separated list of features to enable
- `CARGO_BUILD_JOBS`: Number of parallel jobs

## Building Individual Components

Each component can be built independently by navigating to its directory:

```bash
cd agentflow-rs
cargo build --release
```

## Performance Tips

1. Use `--release` for production builds
2. Set `CARGO_INCREMENTAL=1` for faster development builds
3. Enable LTO for smaller binary sizes in release builds

## Troubleshooting

1. **Dependency Issues**:
   ```bash
   cargo update
   cargo clean
   ```

2. **Linker Errors**:
   - Ensure all system dependencies are installed
   - Check the target platform compatibility

3. **Build Performance**:
   - Use `cargo check` for fast syntax checking
   - Try `cargo build -j $(nproc)` for parallel builds

## License

Apache 2.0
