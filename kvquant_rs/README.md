# KVQuant-RS

[![Crates.io](https://img.shields.io/crates/v/kvquant_rs)](https://crates.io/crates/kvquant_rs)
[![Documentation](https://docs.rs/kvquant_rs/badge.svg)](https://docs.rs/kvquant_rs)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://github.com/zetareticula/zeta-reticula/actions/workflows/rust.yml/badge.svg)](https://github.com/zetareticula/zeta-reticula/actions/workflows/rust.yml)

A high-performance, generic key-value quantization library for Rust, designed for efficient storage and retrieval of quantized neural network weights and activations.

## Features

- **Generic Quantization**: Support for multiple quantization precision levels (8-bit, 4-bit, 2-bit, 1-bit)
- **High Performance**: Built with concurrency in mind using `dashmap` for thread-safe operations
- **Flexible Storage**: Configurable block-based storage with efficient memory usage
- **Reinforcement Learning Integration**: Built-in support for reinforcement learning through the mesolimbic system
- **gRPC Support**: Built-in gRPC server for remote access to the quantization service

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
kvquant_rs = "0.1.0"
```

## Quick Start

```rust
use kvquant_rs::{
    KVQuantConfig, PrecisionLevel, 
    QuantizationResult, QuantizationData,
    KVQuantizer
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new configuration
    let config = KVQuantConfig {
        precision: PrecisionLevel::Int8,
        block_size: 1024,
        cache_capacity: 1000,
    };

    // Create a new quantizer
    let quantizer = KVQuantizer::new(config);

    // Quantize some data
    let data = vec![0.1, 0.2, 0.3, 0.4];
    let quantized = quantizer.quantize(&data)?;
    
    // Dequantize the data
    let dequantized = quantizer.dequantize(&quantized)?;
    
    println!("Original: {:?}", data);
    println!("Dequantized: {:?}", dequantized);
    
    Ok(())
}
```

## Documentation

For detailed documentation, please refer to:

- [API Documentation](https://docs.rs/kvquant_rs)
- [Examples](/examples)
- [Benchmarks](/benches)

## Building from Source

```bash
git clone https://github.com/zetareticula/zeta-reticula.git
cd zeta-reticula/kvquant_rs
cargo build --release
```

## Running Tests

```bash
cargo test --all-features
```

## Benchmarks

Run benchmarks with:

```bash
cargo bench
```

## License

Licensed under the Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgements

- The Zeta Reticula team
- All contributors who have helped improve this project
