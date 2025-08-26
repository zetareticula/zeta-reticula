# Cert-Manager Module

This Terraform module sets up [cert-manager](https://cert-manager.io/) for managing TLS certificates in a Kubernetes cluster. It includes the installation of cert-manager and configuration of Let's Encrypt ClusterIssuers for both production and staging environments.

## Features

- Installs cert-manager using Helm
- Configures Let's Encrypt ClusterIssuers for production and staging
- Supports HTTP01 and DNS01 challenge types
- Configures automatic certificate renewal

## Prerequisites

- Kubernetes cluster (1.16+)
- Helm v3.0.0+
- kubectl configured to communicate with your cluster
- NGINX Ingress Controller installed (if using HTTP01 challenge)
- External DNS configured (if using DNS01 challenge)

## Usage

```hcl
module "cert_manager" {
  source = "./modules/cert-manager"
  
  environment     = "production"
  cluster_name    = "zeta-reticula"
  letsencrypt_email = "admin@example.com"
  
  tags = {
    Environment = "production"
    Terraform   = "true"
  }
}
```

## Inputs

| Name | Description | Type | Default | Required |
|------|-------------|------|---------|:--------:|
| environment | Environment name (e.g., staging, production) | `string` | `"production"` | no |
| cluster_name | Name of the EKS cluster | `string` | `"zeta-reticula"` | no |
| letsencrypt_email | Email address used for Let's Encrypt account registration | `string` | n/a | yes |
| tags | A map of tags to add to all resources | `map(string)` | `{}` | no |

## Outputs

| Name | Description |
|------|-------------|
| cert_manager_namespace | The namespace where cert-manager is installed |
| letsencrypt_prod_issuer_name | Name of the Let's Encrypt production ClusterIssuer |
| letsencrypt_staging_issuer_name | Name of the Let's Encrypt staging ClusterIssuer |

## Using the ClusterIssuers

After applying this module, you can use the ClusterIssuers in your Ingress resources:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: my-app
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"  # or "letsencrypt-staging"
spec:
  tls:
  - hosts:
    - myapp.example.com
    secretName: myapp-tls
  rules:
  - host: myapp.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: my-app
            port:
              number: 80
```

## Troubleshooting

1. **Certificates not being issued**
   - Check cert-manager logs: `kubectl logs -n cert-manager -l app=cert-manager`
   - Check certificate status: `kubectl get certificate,certificaterequest,order,challenge --all-namespaces`

2. **HTTP01 challenges failing**
   - Ensure your Ingress controller is properly configured and accessible
   - Check that the domain resolves to your cluster's external IP

3. **DNS01 challenges failing**
   - Ensure your DNS provider credentials are correctly configured
   - Verify that the DNS records are being created and propagated

## Cleanup

To remove cert-manager and all related resources:

```bash
kubectl delete namespace cert-manager
kubectl delete crd certificaterequests.cert-manager.io certificates.cert-manager.io challenges.acme.cert-manager.io clusterissuers.cert-manager.io issuers.cert-manager.io orders.acme.cert-manager.io
```

## License

This module is licensed under the MIT License. See the LICENSE file for details.
