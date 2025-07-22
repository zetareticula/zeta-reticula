# High Availability Monitoring Setup for P2PStore

This directory contains the configuration for a highly available monitoring setup using Prometheus, Thanos, and Alertmanager.

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Prometheus 1   │◄───►│  Thanos Sidecar 1│◄────┤   MinIO (S3)    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
          ▲                      ▲                       ▲
          │                      │                       │
          ▼                      ▼                       │
┌─────────────────┐     ┌─────────────────┐             │
│  Prometheus 2   │◄───►│  Thanos Sidecar 2│─────────────┘
└─────────────────┘     └─────────────────┘
          ▲                      ▲
          │                      │
          ▼                      ▼
┌─────────────────┐     ┌─────────────────┐
│  Alertmanager   │◄────┤   Thanos Query  │
└─────────────────┘     └─────────────────┘
          ▲                     │
          │                     ▼
          │             ┌─────────────────┐
          └─────────────┤     Grafana     │
                        └─────────────────┘
```

## Components

1. **Prometheus (2 replicas)**
   - Scrapes metrics from services
   - Local storage for short-term metrics (15 days)
   - Configured with external labels for deduplication

2. **Thanos**
   - **Sidecar**: Runs alongside each Prometheus instance, uploads data to object storage
   - **Query**: Provides a unified query interface across all Prometheus instances
   - **Store**: Long-term storage using object storage (MinIO)
   - **Compact**: Compacts and downsamples metrics for long-term storage

3. **Alertmanager**
   - Handles alerts from Prometheus
   - Deduplicates, groups, and routes alerts
   - Supports multiple notification receivers (email, Slack, PagerDuty)

4. **MinIO**
   - S3-compatible object storage for long-term metrics storage
   - Can be replaced with AWS S3, Google Cloud Storage, etc.

5. **Grafana**
   - Visualizes metrics from Thanos Query
   - Pre-configured dashboards

## Getting Started

1. Start the HA monitoring stack:
   ```bash
   docker-compose -f docker-compose-ha.yml up -d
   ```

2. Access the components:
   - Grafana: http://localhost:3000 (admin/admin)
   - Prometheus 1: http://localhost:9091
   - Prometheus 2: http://localhost:9092
   - Thanos Query: http://localhost:10904
   - Alertmanager: http://localhost:9093
   - MinIO: http://localhost:9001 (access_key: minio, secret_key: minio123)

## Configuration

### Prometheus
- Configuration: `prometheus-ha.yml`
- Alert rules: `alert.rules.yml`
- Local retention: 15 days
- Scrape interval: 15s

### Thanos
- Sidecars run alongside each Prometheus instance
- Data is uploaded to MinIO for long-term storage
- Query component provides a unified view

### Alertmanager
- Configuration: `alertmanager.yml`
- Handles alert deduplication and routing
- Supports multiple notification channels

## Scaling

### Adding More Prometheus Instances
1. Add a new service to `docker-compose-ha.yml`
2. Configure the same external labels with a unique replica name
3. Add the new sidecar to the Thanos Query configuration

### Long-term Storage
- By default, metrics are stored in MinIO
- To use a different object storage, update the `minio-bucket.yml` configuration

## Backup and Restore

### Backing Up Configuration
```bash
# Create a backup of all configuration files
cp -r /path/to/monitoring/ha /path/to/backup/ha-config-$(date +%Y%m%d)
```

### Restoring from Backup
```bash
# Stop the services
docker-compose -f docker-compose-ha.yml down

# Restore configuration
cp -r /path/to/backup/ha-config-*/* .

# Start the services
docker-compose -f docker-compose-ha.yml up -d
```

## Monitoring the Monitoring

### Key Metrics to Monitor
- Prometheus uptime and health
- Scrape durations and failures
- Storage utilization
- Alert queue length

### Alerting
Alerts are configured in `alert.rules.yml` and include:
- High request latency
- High error rates
- Service availability
- Resource utilization (CPU, memory, disk)

## Troubleshooting

### Common Issues
1. **Prometheus not scraping targets**
   - Check Prometheus status page: http://localhost:9091/targets
   - Verify network connectivity between Prometheus and targets

2. **Thanos Query not showing data**
   - Check Thanos Query UI: http://localhost:10904/stores
   - Verify sidecars are registered and healthy

3. **Alerts not firing**
   - Check Alertmanager status: http://localhost:9093
   - Verify alert rules are loaded in Prometheus

## Production Considerations

### Security
- Enable authentication for all components
- Use TLS for all communications
- Restrict network access to monitoring components

### Performance
- Adjust resource limits based on load
- Consider sharding Prometheus instances by service/team
- Monitor and tune Thanos components

### High Availability
- Deploy multiple instances of all critical components
- Use a load balancer in front of Thanos Query
- Consider using a managed object storage service for production
