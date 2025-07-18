# P2PStore Monitoring Setup

This directory contains the configuration for monitoring the P2PStore service using Prometheus, Grafana, and Loki.

## Components

1. **Prometheus** - Metrics collection and storage
   - Web UI: http://localhost:9091
   - Scrapes metrics from the P2PStore service

2. **Grafana** - Visualization and dashboards
   - Web UI: http://localhost:3000
   - Default credentials: admin/admin
   - Pre-configured with a P2PStore dashboard

3. **Loki** - Log aggregation
   - Collects and indexes logs from the P2PStore service
   - Integrated with Grafana for log visualization

4. **Promtail** - Log collection agent
   - Collects logs from Docker containers
   - Sends logs to Loki

## Getting Started

1. Start the monitoring stack:
   ```bash
   docker-compose up -d
   ```

2. Access the dashboards:
   - Grafana: http://localhost:3000
   - Prometheus: http://localhost:9091
   - Loki: http://localhost:3100

## Dashboard

The Grafana dashboard includes the following panels:

- **HTTP Request Rate**: Rate of HTTP requests to the P2PStore service
- **HTTP Request Latency**: P95 and P99 latencies for HTTP requests
- **P2P Network Events**: Rate of P2P network events
- **Storage Operations**: Rate of storage operations
- **Logs**: Live log viewer

## Adding Custom Metrics

To add custom metrics to the monitoring:

1. Import the metrics module:
   ```rust
   use crate::monitoring::metrics;
   ```

2. Use the provided metric functions:
   ```rust
   // Record a counter
   metrics::record_network_event("peer_connected", &peer_id);
   
   // Record a histogram
   metrics::record_storage_operation("read", bytes_read);
   ```

## Alerting

To set up alerts:

1. Open Grafana at http://localhost:3000
2. Navigate to Alerting > Alert rules
3. Click "New alert rule"
4. Configure the alert conditions and notification policies

## Troubleshooting

- **No metrics showing?**
  - Check if the P2PStore service is running and exposing metrics on port 9090
  - Verify Prometheus is scraping the correct target (check http://localhost:9091/targets)

- **No logs in Grafana?**
  - Check if Loki is running and accessible
  - Verify Promtail is collecting logs (check its logs with `docker logs p2pstore_promtail_1`)

## Cleanup

To stop and remove all monitoring containers:

```bash
docker-compose down -v
```

This will remove all containers and volumes, including stored metrics and logs.
