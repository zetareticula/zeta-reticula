# Ingress for Prometheus
resource "kubernetes_ingress_v1" "prometheus" {
  metadata {
    name      = "prometheus"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
    annotations = {
      "kubernetes.io/ingress.class"           = "nginx"
      "nginx.ingress.kubernetes.io/ssl-redirect" = "true"
      "cert-manager.io/cluster-issuer"        = var.environment == "production" ? "letsencrypt-prod" : "letsencrypt-staging"
      "nginx.ingress.kubernetes.io/auth-type"   = "basic"
      "nginx.ingress.kubernetes.io/auth-secret" = "prometheus-basic-auth"
      "nginx.ingress.kubernetes.io/auth-realm"  = "Authentication Required"
    }
  }

  spec {
    tls {
      hosts       = ["prometheus.${var.environment}.${var.cluster_name}"]
      secret_name = "prometheus-tls"
    }

    rule {
      host = "prometheus.${var.environment}.${var.cluster_name}"
      http {
        path {
          path = "/"
          backend {
            service {
              name = "kube-prometheus-stack-prometheus"
              port {
                number = 9090
              }
            }
          }
        }
      }
    }
  }
  
  depends_on = [
    helm_release.kube_prometheus_stack
  ]
}

# Ingress for Grafana
resource "kubernetes_ingress_v1" "grafana" {
  metadata {
    name      = "grafana"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
    annotations = {
      "kubernetes.io/ingress.class"           = "nginx"
      "nginx.ingress.kubernetes.io/ssl-redirect" = "true"
      "cert-manager.io/cluster-issuer"        = var.environment == "production" ? "letsencrypt-prod" : "letsencrypt-staging"
    }
  }

  spec {
    tls {
      hosts       = ["grafana.${var.environment}.${var.cluster_name}"]
      secret_name = "grafana-tls"
    }

    rule {
      host = "grafana.${var.environment}.${var.cluster_name}"
      http {
        path {
          path = "/"
          backend {
            service {
              name = "kube-prometheus-stack-grafana"
              port {
                number = 80
              }
            }
          }
        }
      }
    }
  }
  
  depends_on = [
    helm_release.kube_prometheus_stack
  ]
}

# Ingress for Alertmanager
resource "kubernetes_ingress_v1" "alertmanager" {
  count = var.alertmanager_enabled ? 1 : 0
  
  metadata {
    name      = "alertmanager"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
    annotations = {
      "kubernetes.io/ingress.class"           = "nginx"
      "nginx.ingress.kubernetes.io/ssl-redirect" = "true"
      "cert-manager.io/cluster-issuer"        = var.environment == "production" ? "letsencrypt-prod" : "letsencrypt-staging"
      "nginx.ingress.kubernetes.io/auth-type"   = "basic"
      "nginx.ingress.kubernetes.io/auth-secret" = "alertmanager-basic-auth"
      "nginx.ingress.kubernetes.io/auth-realm"  = "Authentication Required"
    }
  }

  spec {
    tls {
      hosts       = ["alertmanager.${var.environment}.${var.cluster_name}"]
      secret_name = "alertmanager-tls"
    }

    rule {
      host = "alertmanager.${var.environment}.${var.cluster_name}"
      http {
        path {
          path = "/"
          backend {
            service {
              name = "kube-prometheus-stack-alertmanager"
              port {
                number = 9093
              }
            }
          }
        }
      }
    }
  }
  
  depends_on = [
    helm_release.kube_prometheus_stack
  ]
}
