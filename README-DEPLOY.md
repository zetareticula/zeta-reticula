# Zeta Reticula Deployment Guide

This guide will help you set up and run the Zeta Reticula system with all its components.

## Prerequisites

1. Docker and Docker Compose installed on your system
2. At least 64GB of RAM and 100GB of free disk space (for model storage and processing)
3. NVIDIA GPU with CUDA support (recommended for optimal performance)

## Quick Start

1. **Clone the repository**
   ```bash
   git clone https://github.com/zetareticula/zeta-reticula.git
   cd zeta-reticula
   ```

2. **Initialize the system**
   ```bash
   ./init_system.sh
   ```

3. **Download the model weights**
   Place your model files in the `models/llama2-7b` directory. The expected structure is:
   ```
   models/
   └── llama2-7b/
       ├── config.json
       ├── tokenizer.json
       ├── tokenizer_config.json
       ├── model.safetensors
       └── ... other model files
   ```

4. **Start the system**
   ```bash
   docker-compose -f docker-compose.full.yml up -d
   ```

5. **Access the services**
   - API Gateway: http://localhost:3000
   - Prometheus: http://localhost:9090
   - Grafana: http://localhost:3001 (default credentials: admin/admin)
   - Salience Engine: http://localhost:8080

## System Architecture

The Zeta Reticula system consists of the following main components:

1. **Salience Engine**: Core inference engine with mesolimbic integration
2. **NS Router**: Distributed routing service
3. **AgentFlow**: Workflow management and orchestration
4. **LLM Service**: Model serving and inference
5. **KV Quant**: Key-value store with quantization support
6. **Attention Store**: Manages attention mechanisms and caching
7. **Zeta Vault**: Integration layer between components
8. **API Gateway**: Unified API endpoint for clients

## Configuration

### Environment Variables

Key environment variables can be configured in the `docker-compose.full.yml` file:

- `QUANTIZATION_BITS`: Comma-separated list of bit-widths for quantization (e.g., "1,2,4,8,16")
- `MODEL_PATH`: Path to the model files
- `KV_STORE_ENABLED`: Enable/disable key-value store
- `MESOLIMBIC_ENGINE_ENABLED`: Enable/disable mesolimbic engine

### Monitoring

The system includes Prometheus and Grafana for monitoring. Pre-configured dashboards are available in the `monitoring/` directory.

## Advanced Usage

### Running Quantization

To run quantization on a model:

```bash
docker-compose exec quantize-cli quantize-cli --model /app/models/llama2-7b --bits 1,2,4,8,16 --output /app/results
```

### Scaling Services

To scale services:

```bash
docker-compose up -d --scale llm-service=2 --scale salience-engine=3
```

## Troubleshooting

1. **Out of Memory Errors**
   - Increase Docker's memory allocation
   - Reduce the number of concurrent operations
   - Use smaller batch sizes

2. **Model Loading Issues**
   - Verify model files are in the correct location
   - Check file permissions
   - Ensure the model format is supported

3. **Performance Optimization**
   - Enable GPU acceleration
   - Adjust batch sizes based on available memory
   - Use appropriate quantization levels for your hardware

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
