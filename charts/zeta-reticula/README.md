# Zeta Reticula Helm Chart

This Helm chart deploys the Zeta Reticula unified CLI application to a Kubernetes cluster.

## Installation

### From Source
```bash
helm install zeta-reticula ./charts/zeta-reticula
```

### With Production Values
```bash
helm install zeta-reticula ./charts/zeta-reticula \
  -f ./charts/zeta-reticula/values-production.yaml
```

## Configuration

The following table lists the configurable parameters of the Zeta Reticula chart and their default values.

| Parameter | Description | Default |
|-----------|-------------|---------|
| `zetaReticula.enabled` | Enable/disable the Zeta Reticula deployment | `true` |
| `zetaReticula.replicas` | Number of replicas | `3` |
| `zetaReticula.image.repository` | Docker image repository | `zetareticula/zeta-reticula` |
| `zetaReticula.image.tag` | Docker image tag | `latest` |
| `zetaReticula.resources.limits.cpu` | CPU resource limit | `1000m` |
| `zetaReticula.resources.limits.memory` | Memory resource limit | `1Gi` |
| `ingress.enabled` | Enable/disable ingress | `true` |
| `ingress.hosts[0]` | Hostname for ingress | `zeta-reticula.local` |
| `persistence.enabled` | Enable/disable persistent volumes | `false` |
| `autoscaling.enabled` | Enable/disable horizontal pod autoscaler | `false` |

## Production Deployment

For production deployments, use the production values file:

```bash
helm install zeta-reticula ./charts/zeta-reticula \
  -f ./charts/zeta-reticula/values-production.yaml \
  --namespace production \
  --create-namespace
```

### Production Features

- **High Availability**: Multiple replicas with pod disruption budgets
- **Resource Management**: Optimized CPU/memory limits and requests
- **Security**: Non-root user, read-only root filesystem, security contexts
- **Monitoring**: Prometheus metrics, health checks, and alerting rules
- **Networking**: Network policies, service mesh integration
- **Storage**: Persistent volumes for data persistence
- **Auto-scaling**: Horizontal Pod Autoscaler for dynamic scaling

## Development

### Testing Locally

```bash
# Test template rendering
helm template zeta-reticula ./charts/zeta-reticula \
  -f ./charts/zeta-reticula/values.yaml

# Lint the chart
helm lint ./charts/zeta-reticula

# Package the chart
helm package ./charts/zeta-reticula
```

### Custom Values

Create a custom `values.yaml` file:

```yaml
zetaReticula:
  replicas: 5
  image:
    tag: "v1.0.0"
  resources:
    limits:
      cpu: "2000m"
      memory: "2Gi"
  config:
    RUST_LOG: "debug"

ingress:
  hosts:
    - host: my-zeta-reticula.example.com
```

Then install with:

```bash
helm install zeta-reticula ./charts/zeta-reticula -f custom-values.yaml
```

## Components

- **Deployment**: Unified Zeta Reticula application
- **Service**: ClusterIP service for internal communication
- **Ingress**: HTTP ingress for external access
- **ConfigMap**: Application configuration
- **Secret**: Sensitive configuration data
- **ServiceAccount**: Kubernetes service account
- **NetworkPolicy**: Network security policies
- **PodDisruptionBudget**: High availability configuration

## Troubleshooting

### Common Issues

1. **Image Pull Errors**: Ensure the Docker image is built and pushed to the registry
2. **Resource Constraints**: Check resource limits and requests match cluster capacity
3. **Network Issues**: Verify ingress configuration and DNS settings
4. **Permission Issues**: Check RBAC permissions and service account configuration

### Debugging Commands

```bash
# Check pod status
kubectl get pods -l app.kubernetes.io/name=zeta-reticula

# View pod logs
kubectl logs -l app.kubernetes.io/name=zeta-reticula

# Check service endpoints
kubectl get endpoints -l app.kubernetes.io/name=zeta-reticula

# Debug ingress
kubectl describe ingress zeta-reticula
```

## Upgrading

To upgrade an existing installation:

```bash
helm upgrade zeta-reticula ./charts/zeta-reticula \
  -f ./charts/zeta-reticula/values-production.yaml
```

## Uninstallation

To remove the deployment:

```bash
helm uninstall zeta-reticula
```

Note: This will remove all Kubernetes resources but persistent volumes will remain.
