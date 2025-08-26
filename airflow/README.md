# Zeta Reticula Airflow

This directory contains the Airflow setup for orchestrating the Zeta Reticula inference pipeline.

## Prerequisites

- Docker and Docker Compose
- Kubernetes cluster (for Kubernetes-based tasks)
- `kubectl` configured to access your cluster

## Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/zetareticula/zeta-reticula.git
   cd zeta-reticula/airflow
   ```

2. **Set up environment variables**
   ```bash
   cp .env.example .env
   # Edit .env file with your configuration
   ```

3. **Initialize and start Airflow**
   ```bash
   ./setup_airflow.sh
   ```
   This will:
   - Create necessary directories
   - Initialize the Airflow database
   - Start all required services
   - Set up the Kubernetes connection

4. **Access the Airflow UI**
   - Open http://localhost:8080 in your browser
   - Login with username: `admin` / password: `admin`

## DAGs

The following DAGs are available:

### `zeta_reticula_workflow`

Orchestrates the Zeta Reticula model inference pipeline with the following steps:

1. Check model registry availability
2. Get latest model version
3. Ingest model
4. Quantize model
5. Validate model
6. Deploy model
7. Test model
8. Send notification

## Configuration

### Environment Variables

- `ENVIRONMENT`: Deployment environment (development/staging/production)
- `K8S_NAMESPACE`: Kubernetes namespace for running tasks (default: zeta-reticula)
- `MODEL_REGISTRY`: Container registry for model images (default: zetareticula)

### Connections

- `kubernetes_default`: Connection to the Kubernetes cluster
- `postgres`: Connection to the PostgreSQL database

## Development

### Adding New DAGs

1. Add your DAG file to the `dags/` directory
2. The file will be automatically picked up by Airflow

### Adding Dependencies

1. Add Python packages to `requirements.txt`
2. Rebuild the Docker image:
   ```bash
   docker-compose build --no-cache
   docker-compose up -d
   ```

## Troubleshooting

### View Logs

```bash
# Airflow webserver logs
docker-compose logs -f airflow-webserver

# Scheduler logs
docker-compose logs -f airflow-scheduler

# Worker logs
docker-compose logs -f airflow-worker
```

### Reset Airflow Database

```bash
docker-compose down --volumes --remove-orphans
./setup_airflow.sh
```

## Monitoring

Airflow provides built-in monitoring through the web UI. Additional monitoring can be set up using:

- Prometheus metrics endpoint: `http://localhost:8080/metrics`
- StatsD metrics
- Logging to external services

## Security

- Change the default admin password after first login
- Use HTTPS in production
- Configure authentication backends as needed
- Regularly update dependencies for security patches
