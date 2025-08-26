# Zeta Reticula - Scripts Directory

This directory contains various utility scripts for managing the Zeta Reticula system.

## Available Scripts

### Management Scripts

- `manage.sh` - Main management script for the Zeta Reticula system
  ```bash
  # Start all services
  ./scripts/manage.sh start
  
  # Stop all services
  ./scripts/manage.sh stop
  
  # Restart all services
  ./scripts/manage.sh restart
  
  # Show status of all services
  ./scripts/manage.sh status
  
  # View logs
  ./scripts/manage.sh logs [service]
  
  # Clean up temporary files
  ./scripts/manage.sh clean
  
  # Update the system
  ./scripts/manage.sh update
  
  # Show help
  ./scripts/manage.sh help
  ```

### Individual Service Scripts

- `start_salience_engine.sh` - Start the salience-engine and all its dependencies
  ```bash
  ./scripts/start_salience_engine.sh
  ```

- `stop_services.sh` - Stop all running services
  ```bash
  ./scripts/stop_services.sh
  ```

- `status.sh` - Show status of all services and system resources
  ```bash
  ./scripts/status.sh
  ```

### Utility Scripts

- `check_requirements.sh` - Check system requirements and dependencies
  ```bash
  ./scripts/check_requirements.sh
  ```

- `cleanup.sh` - Clean up temporary files and resources
  ```bash
  ./scripts/cleanup.sh
  ```

- `download_model.sh` - Download and prepare ML models
  ```bash
  # Download a model
  ./scripts/download_model.sh --model llama-2-7b
  
  # Download and quantize a model
  ./scripts/download_model.sh --model llama-2-7b --quantize --bits 4 --group-size 64
  ```

## Environment Variables

All scripts read configuration from the `.env` file in the project root. Make sure to create this file from `.env.example` and update it with your configuration.

## Logs

Logs for each service are stored in the `logs` directory in the project root. Each service has its own log file:

- `distributed-store.log` - Distributed store logs
- `master-service.log` - Master service logs
- `agentflow.log` - AgentFlow logs
- `salience-engine.log` - Salience engine logs

## Troubleshooting

If you encounter any issues:

1. Check the logs for the relevant service
2. Run `./scripts/check_requirements.sh` to verify system requirements
3. Try cleaning up and restarting the system:
   ```bash
   ./scripts/manage.sh clean
   ./scripts/manage.sh start
   ```
4. If the issue persists, please open an issue on GitHub with the relevant logs and system information.
