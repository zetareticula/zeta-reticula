# Create cert-manager namespace
resource "kubernetes_namespace" "cert_manager" {
  metadata {
    name = "cert-manager"
    labels = {
      "cert-manager.io/disable-validation" = "true"
    }
  }
}

# Install cert-manager CRDs
resource "kubectl_manifest" "cert_manager_crds" {
  for_each = toset([
    "https://github.com/cert-manager/cert-manager/releases/download/v1.11.0/cert-manager.crds.yaml"
  ])
  
  yaml_body = file("${path.module}/crds/cert-manager.crds.yaml")
  
  depends_on = [kubernetes_namespace.cert_manager]
}

# Install cert-manager using Helm
resource "helm_release" "cert_manager" {
  name       = "cert-manager"
  namespace  = kubernetes_namespace.cert_manager.metadata[0].name
  repository = "https://charts.jetstack.io"
  chart      = "cert-manager"
  version    = "v1.11.0"
  
  set {
    name  = "installCRDs"
    value = "false" # We install CRDs separately
  }
  
  set {
    name  = "prometheus.enabled"
    value = "true"
  }
  
  set {
    name  = "prometheus.servicemonitor.enabled"
    value = "true"
  }
  
  set {
    name  = "prometheus.servicemonitor.namespace"
    value = "monitoring"
  }
  
  depends_on = [kubectl_manifest.cert_manager_crds]
}

# Create ClusterIssuer for Let's Encrypt production
resource "kubectl_manifest" "letsencrypt_prod_issuer" {
  yaml_body = <<YAML
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: ${var.letsencrypt_email}
    privateKeySecretRef:
      name: letsencrypt-prod-account-key
    solvers:
    - http01:
        ingress:
          class: nginx
      selector: {}
YAML

  depends_on = [helm_release.cert_manager]
}

# Create ClusterIssuer for Let's Encrypt staging (for testing)
resource "kubectl_manifest" "letsencrypt_staging_issuer" {
  yaml_body = <<YAML
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-staging
spec:
  acme:
    server: https://acme-staging-v02.api.letsencrypt.org/directory
    email: ${var.letsencrypt_email}
    privateKeySecretRef:
      name: letsencrypt-staging-account-key
    solvers:
    - http01:
        ingress:
          class: nginx
      selector: {}
YAML

  depends_on = [helm_release.cert_manager]
}
