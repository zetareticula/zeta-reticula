# Generate a random password for basic auth if not provided
resource "random_password" "basic_auth" {
  count   = var.basic_auth_enabled && var.basic_auth_password == "" ? 1 : 0
  length  = 16
  special = true
}

# Generate bcrypt hash of the password
resource "random_password" "bcrypt_hash" {
  count   = var.basic_auth_enabled ? 1 : 0
  length  = 16
  special = true
  
  # Use a fixed special character set to ensure the hash is valid
  override_special = "!@#$%&*()-_=+[]{}<>:?"
}

# Create Kubernetes secret for Prometheus basic auth
resource "kubernetes_secret" "prometheus_basic_auth" {
  count = var.basic_auth_enabled ? 1 : 0
  
  metadata {
    name      = "prometheus-basic-auth"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
  }

  data = {
    auth = "${var.basic_auth_username}:{SHA}${base64encode(sha256("${var.basic_auth_password != "" ? var.basic_auth_password : random_password.basic_auth[0].result}"))}"
  }

  type = "Opaque"
  
  depends_on = [
    kubernetes_namespace.monitoring
  ]
}

# Create Kubernetes secret for Alertmanager basic auth
resource "kubernetes_secret" "alertmanager_basic_auth" {
  count = var.alertmanager_enabled && var.basic_auth_enabled ? 1 : 0
  
  metadata {
    name      = "alertmanager-basic-auth"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
  }

  data = {
    auth = "${var.basic_auth_username}:{SHA}${base64encode(sha256("${var.basic_auth_password != "" ? var.basic_auth_password : random_password.basic_auth[0].result}"))}"
  }

  type = "Opaque"
  
  depends_on = [
    kubernetes_namespace.monitoring
  ]
}
