# Monitoring Module

This Terraform module deploys a comprehensive monitoring stack for the Zeta Reticula master service on Kubernetes, including:

- **Prometheus** - Metrics collection and alerting
- **Alertmanager** - Alert management and routing
- **Grafana** - Visualization and dashboards
- **Loki** - Log aggregation (optional)
- **Promtail** - Log collection (if Loki is enabled)

## Prerequisites

- Kubernetes cluster (EKS, GKE, AKS, or any other CNCF-certified Kubernetes distribution)
- `kubectl` configured to communicate with your cluster
- `helm` installed and configured
- `terraform` v1.0.0 or later

## Usage

```hcl
module "monitoring" {
  source = "git::https://github.com/your-org/terraform-modules.git//modules/monitoring?ref=v1.0.0"

  environment = "production"
  cluster_name = "zeta-reticula"
  
  # Enable/disable Loki (log aggregation)
  enable_loki = true
  
  # Prometheus configuration
  prometheus_storage_class = "gp2"
  prometheus_storage_size = "50Gi"
  prometheus_retention = "15d"
  
  # Loki configuration (if enabled)
  loki_storage_class = "gp2"
  loki_storage_size = "100Gi"
  
  # Alertmanager configuration
  alertmanager_enabled = true
  
  # Grafana configuration
  grafana_admin_password = "your-secure-password"
}
```

## Inputs

| Name | Description | Type | Default | Required |
|------|-------------|------|---------|:--------:|
| environment | Environment name (e.g., staging, production) | `string` | n/a | yes |
| cluster_name | Name of the Kubernetes cluster | `string` | n/a | yes |
| enable_loki | Enable Loki for log aggregation | `bool` | `true` | no |
| prometheus_storage_class | Storage class for Prometheus persistent volume | `string` | `"gp2"` | no |
| prometheus_storage_size | Storage size for Prometheus persistent volume | `string` | `"50Gi"` | no |
| prometheus_retention | Retention period for Prometheus metrics | `string` | `"15d"` | no |
| loki_storage_class | Storage class for Loki persistent volume | `string` | `"gp2"` | no |
| loki_storage_size | Storage size for Loki persistent volume | `string` | `"100Gi"` | no |
| alertmanager_enabled | Enable Alertmanager | `bool` | `true` | no |
| grafana_admin_password | Admin password for Grafana | `string` | `"admin"` | no |

## Outputs

| Name | Description |
|------|-------------|
| namespace | The name of the monitoring namespace |
| prometheus_service_name | The name of the Prometheus service |
| prometheus_service_port | The port of the Prometheus service |
| alertmanager_service_name | The name of the Alertmanager service |
| alertmanager_service_port | The port of the Alertmanager service |
| grafana_service_name | The name of the Grafana service |
| grafana_service_port | The port of the Grafana service |
| loki_service_name | The name of the Loki service (if enabled) |
| loki_service_port | The port of the Loki service |
| prometheus_ingress_host | The hostname for the Prometheus ingress |
| grafana_ingress_host | The hostname for the Grafana ingress |
| alertmanager_ingress_host | The hostname for the Alertmanager ingress |
| loki_ingress_host | The hostname for the Loki ingress (if enabled) |

## Accessing the Monitoring Stack

### Port-forwarding

You can access the monitoring stack using port-forwarding:

```bash
# Prometheus
kubectl port-forward -n monitoring svc/prometheus-operated 9090:9090

# Grafana
kubectl port-forward -n monitoring svc/kube-prometheus-stack-grafana 3000:80

# Alertmanager (if enabled)
kubectl port-forward -n monitoring svc/alertmanager-operated 9093:9093

# Loki (if enabled)
kubectl port-forward -n monitoring svc/loki 3100:3100
```

### Get Grafana Admin Password

```bash
kubectl get secret --namespace monitoring kube-prometheus-stack-grafana -o jsonpath='{.data.admin-password}' | base64 --decode ; echo
```

## Customizing Dashboards and Alerts

### Adding Custom Dashboards

1. Create a ConfigMap with your Grafana dashboard JSON
2. Label it with `grafana_dashboard: "1"`
3. Place it in the `monitoring` namespace

Example:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: my-custom-dashboard
  namespace: monitoring
  labels:
    grafana_dashboard: "1"
data:
  my-dashboard.json: |-
    {
      "annotations": {
        "description": "My Custom Dashboard",
        "version": "1.0"
      },
      "editable": true,
      "gnetId": null,
      "graphTooltip": 0,
      "links": [],
      "panels": [],
      "schemaVersion": 27,
      "style": "dark",
      "tags": [],
      "templating": {
        "list": []
      },
      "time": {
        "from": "now-6h",
        "to": "now"
      },
      "timepicker": {},
      "timezone": "browser",
      "title": "My Custom Dashboard",
      "uid": "my-custom-dashboard"
    }
```

### Adding Custom Alert Rules

1. Create a PrometheusRule resource in the `monitoring` namespace
2. Use the `prometheus: k8s` label for standard alerts

Example:

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: my-custom-rules
  namespace: monitoring
  labels:
    app: kube-prometheus-stack
    release: kube-prometheus-stack
spec:
  groups:
  - name: my-custom-rules
    rules:
    - alert: MyCustomAlert
      expr: up == 0
      for: 5m
      labels:
        severity: critical
      annotations:
        summary: "Instance {{ $labels.instance }} down"
        description: "{{ $labels.instance }} of job {{ $labels.job }} has been down for more than 5 minutes."
```

## Upgrading

To upgrade the monitoring stack, update the Helm chart versions in the module and run `terraform apply`.

## Cleanup

To completely remove the monitoring stack:

```bash
# Delete all resources in the monitoring namespace
kubectl delete all --all -n monitoring

# Delete the monitoring namespace
kubectl delete namespace monitoring

# Delete any remaining CRDs
kubectl delete crd alertmanagerconfigs.monitoring.coreos.com
kubectl delete crd alertmanagers.monitoring.coreos.com
kubectl delete crd podmonitors.monitoring.coreos.com
kubectl delete crd probes.monitoring.coreos.com
kubectl delete crd prometheuses.monitoring.coreos.com
kubectl delete crd prometheusrules.monitoring.coreos.com
kubectl delete crd servicemonitors.monitoring.coreos.com
kubectl delete crd thanosrulers.monitoring.coreos.com
```

## Troubleshooting

### Prometheus Pods in CrashLoopBackOff

Check the logs:

```bash
kubectl logs -n monitoring -l app.kubernetes.io/name=prometheus
```

Common issues:
- Storage class not available
- Insufficient storage size
- Permission issues with persistent volumes

### Grafana Login Issues

If you can't log in to Grafana:

1. Verify the admin password:
   ```bash
   kubectl get secret --namespace monitoring kube-prometheus-stack-grafana -o jsonpath='{.data.admin-password}' | base64 --decode ; echo
   ```

2. Check the Grafana pod logs:
   ```bash
   kubectl logs -n monitoring -l app.kubernetes.io/name=grafana
   ```

### Loki Not Collecting Logs

1. Check Promtail logs:
   ```bash
   kubectl logs -n monitoring -l app.kubernetes.io/name=promtail
   ```

2. Verify Loki is running:
   ```bash
   kubectl get pods -n monitoring -l app.kubernetes.io/name=loki
   ```

3. Check Loki logs:
   ```bash
   kubectl logs -n monitoring -l app.kubernetes.io/name=loki
   ```

## License

This module is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
