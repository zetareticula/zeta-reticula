# Zeta Reticula Quantize CLI

A powerful command-line tool for quantizing, optimizing, and running inference on large language models (LLMs) with neurosymbolic salience analysis and routing.

## Features

- **Model Quantization**: Reduce model size with various bit-widths (4, 8, 16, 32 bits)
- **Salience-Aware Quantization**: Optimize quantization based on token importance
- **Neurosymbolic Routing**: Intelligent routing of inference requests
- **Model Optimization**: KV cache optimization and other performance improvements
- **Format Conversion**: Convert between different model formats (GGUF, SafeTensors, etc.)

## Installation

### Prerequisites

- Rust (latest stable version)
- Cargo (Rust's package manager)
- OpenBLAS (recommended for better performance)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/zeta-reticula.git
add ucd zeta-reticula

# Build the quantize-cli
cargo build --release --bin quantize-cli

# The binary will be available at:
# ./target/release/quantize-cli
```

## Usage

### Quantize a Model

```bash
# Basic quantization (8-bit by default)
quantize-cli quantize --input model.gguf --output quantized_model.gguf

# 4-bit quantization with salience analysis
quantize-cli quantize --input model.gguf --output quantized_4bit.gguf --bits 4 --use-salience

# Specify output directory
quantize-cli quantize -i model.gguf -o ./quantized -b 8
```

### Run Inference

```bash
# Basic inference
quantize-cli infer --model model.gguf --input "Your prompt here"

# With neurosymbolic routing
quantize-cli infer -m model.gguf -i "Your prompt" --use-ns-router

# Limit output tokens
quantize-cli infer -m model.gguf -i "Your prompt" --max-tokens 50
```

### Optimize a Model

```bash
# Basic optimization
quantize-cli optimize --model model.gguf --output optimized.gguf

# With KV cache optimization
quantize-cli optimize -m model.gguf -o optimized.gguf --use-kv-cache
```

### Convert Model Formats

```bash
# Convert to GGUF format
quantize-cli convert --input model.bin --output model.gguf --format gguf

# Convert to SafeTensors format
quantize-cli convert -i model.gguf -o model.safetensors -f safetensors
```

## Advanced Usage

### Verbose Output

Add the `-v` or `--verbose` flag to enable debug logging:

```bash
quantize-cli -v quantize -i model.gguf -o quantized.gguf
```

### Output Format

Specify output format with `--format` (default: json):

```bash
quantize-cli --format yaml infer -m model.gguf -i "Prompt"
```

## Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) for details.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with ❤️ by the Zeta Reticula team
- Uses [llm-rs](https://github.com/rustformers/llm) for model operations
- Inspired by the latest research in model quantization and optimization
