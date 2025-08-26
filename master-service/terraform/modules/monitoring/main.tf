# Configure providers
terraform {
  required_providers {
    random = {
      source  = "hashicorp/random"
      version = "~> 3.0"
    }
  }
}

# Create Kubernetes namespace for monitoring
resource "kubernetes_namespace" "monitoring" {
  metadata {
    name = "monitoring"
    
    labels = {
      name        = "monitoring"
      environment = var.environment
    }
  }
}

# Install kube-prometheus-stack using Helm
resource "helm_release" "kube_prometheus_stack" {
  name       = "kube-prometheus-stack"
  namespace  = kubernetes_namespace.monitoring.metadata[0].name
  repository = "https://prometheus-community.github.io/helm-charts"
  chart      = "kube-prometheus-stack"
  version    = "45.0.0"  # Use the latest stable version
  wait       = true
  
  values = [
    templatefile("${path.module}/values/prometheus-values.yaml", {
      environment = var.environment
      storage_class = var.prometheus_storage_class
      storage_size  = var.prometheus_storage_size
      retention     = var.prometheus_retention
      alertmanager_enabled = var.alertmanager_enabled
      grafana_admin_password = var.grafana_admin_password
    })
  ]
  
  set {
    name  = "prometheus.prometheusSpec.podMonitorSelectorNilUsesHelmValues"
    value = "false"
  }
  
  set {
    name  = "prometheus.prometheusSpec.serviceMonitorSelectorNilUsesHelmValues"
    value = "false"
  }
  
  depends_on = [
    kubernetes_namespace.monitoring
  ]
}

# Install Loki for logs
resource "helm_release" "loki" {
  count      = var.enable_loki ? 1 : 0
  name       = "loki"
  namespace  = kubernetes_namespace.monitoring.metadata[0].name
  repository = "https://grafana.github.io/helm-charts"
  chart      = "loki-stack"
  version    = "2.9.10"  # Use the latest stable version
  
  set {
    name  = "loki.persistence.enabled"
    value = "true"
  }
  
  set {
    name  = "loki.persistence.size"
    value = var.loki_storage_size
  }
  
  set {
    name  = "loki.persistence.storageClassName"
    value = var.loki_storage_class
  }
  
  set {
    name  = "promtail.enabled"
    value = "true"
  }
  
  depends_on = [
    kubernetes_namespace.monitoring
  ]
}

# Install Promtail for log collection
resource "kubernetes_daemonset" "promtail" {
  count = var.enable_loki ? 1 : 0
  
  metadata {
    name      = "promtail"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
    
    labels = {
      app     = "promtail"
      release = "loki"
    }
  }
  
  spec {
    selector {
      match_labels = {
        app     = "promtail"
        release = "loki"
      }
    }
    
    template {
      metadata {
        labels = {
          app     = "promtail"
          release = "loki"
        }
      }
      
      spec {
        container {
          name  = "promtail"
          image = "grafana/promtail:2.8.0"
          
          args = [
            "-config.file=/etc/promtail/promtail.yaml"
          ]
          
          volume_mount {
            name       = "config"
            mount_path = "/etc/promtail"
          }
          
          volume_mount {
            name       = "run"
            mount_path = "/var/run/promtail"
          }
          
          volume_mount {
            name       = "docker"
            mount_path = "/var/lib/docker"
            read_only  = true
          }
          
          volume_mount {
            name       = "pods"
            mount_path = "/var/log/pods"
            read_only  = true
          }
        }
        
        volume {
          name = "config"
          
          config_map {
            name = "promtail-config"
          }
        }
        
        volume {
          name = "run"
          
          host_path {
            path = "/var/run/promtail"
            type = "DirectoryOrCreate"
          }
        }
        
        volume {
          name = "docker"
          
          host_path {
            path = "/var/lib/docker"
          }
        }
        
        volume {
          name = "pods"
          
          host_path {
            path = "/var/log/pods"
          }
        }
        
        service_account_name            = "promtail"
        automount_service_account_token = true
      }
    }
  }
  
  depends_on = [
    kubernetes_namespace.monitoring,
    helm_release.loki
  ]
}

# Create Promtail ConfigMap
resource "kubernetes_config_map" "promtail_config" {
  count = var.enable_loki ? 1 : 0
  
  metadata {
    name      = "promtail-config"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
  }
  
  data = {
    "promtail.yaml" = <<-EOT
      server:
        http_listen_port: 9080
        grpc_listen_port: 0
      
      positions:
        filename: /var/run/promtail/positions.yaml
      
      clients:
        - url: http://loki:3100/loki/api/v1/push
      
      scrape_configs:
        - job_name: kubernetes-pods
          kubernetes_sd_configs:
            - role: pod
          relabel_configs:
            - source_labels: [__meta_kubernetes_pod_annotation_promtail_skip]
              regex: true
              action: drop
            - source_labels: [__meta_kubernetes_pod_annotation_kubernetes_io_config_mirror]
              regex: .+
              action: drop
            - source_labels: [__meta_kubernetes_pod_phase]
              regex: ^(Failed|Succeeded)$
              action: drop
            - source_labels: [__meta_kubernetes_pod_container_name]
              target_label: container
            - source_labels: [__meta_kubernetes_pod_node_name]
              target_label: node
            - source_labels: [__meta_kubernetes_pod_name]
              target_label: pod
            - source_labels: [__meta_kubernetes_pod_label_app]
              target_label: app
            - source_labels: [__meta_kubernetes_namespace]
              target_label: namespace
            - source_labels: [__meta_kubernetes_pod_container_name]
              target_label: container_name
            - target_label: host
              replacement: ${var.environment}
            - target_label: job
              replacement: ${var.environment}/{{ .Values.namespace }}/{{ .Values.pod }}
      
      pipeline_stages:
        - docker: {}
        - match:
            selector: '{app=~".+"}'
            stages:
              - regex:
                  expression: '^(?P<timestamp>\S+) (?P<stream>stdout|stderr) (?P<flags>\S+) (?P<content>.*)'
              - timestamp:
                  source: timestamp
                  format: RFC3339Nano
              - labels:
                  stream:
                  flags:
        - output:
            source: content
    EOT
  }
  
  depends_on = [
    kubernetes_namespace.monitoring
  ]
}

# Create Promtail ServiceAccount
resource "kubernetes_service_account" "promtail" {
  count = var.enable_loki ? 1 : 0
  
  metadata {
    name      = "promtail"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
  }
  
  depends_on = [
    kubernetes_namespace.monitoring
  ]
}

# Create ClusterRole for Promtail
resource "kubernetes_cluster_role" "promtail" {
  count = var.enable_loki ? 1 : 0
  
  metadata {
    name = "promtail"
  }
  
  rule {
    api_groups = [""]
    resources  = ["nodes", "nodes/proxy", "services", "endpoints", "pods"]
    verbs      = ["get", "list", "watch"]
  }
  
  depends_on = [
    kubernetes_namespace.monitoring
  ]
}

# Create ClusterRoleBinding for Promtail
resource "kubernetes_cluster_role_binding" "promtail" {
  count = var.enable_loki ? 1 : 0
  
  metadata {
    name = "promtail"
  }
  
  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.promtail[0].metadata[0].name
  }
  
  subject {
    kind      = "ServiceAccount"
    name      = kubernetes_service_account.promtail[0].metadata[0].name
    namespace = kubernetes_namespace.monitoring.metadata[0].name
  }
  
  depends_on = [
    kubernetes_cluster_role.promtail,
    kubernetes_service_account.promtail
  ]
}

# Create ServiceMonitor for Prometheus to scrape itself
resource "kubernetes_manifest" "prometheus_servicemonitor" {
  manifest = {
    "apiVersion" = "monitoring.coreos.com/v1"
    "kind"       = "ServiceMonitor"
    "metadata" = {
      "name"      = "kube-prometheus-stack-prometheus"
      "namespace" = kubernetes_namespace.monitoring.metadata[0].name
      "labels" = {
        "release" = "kube-prometheus-stack"
      }
    }
    "spec" = {
      "endpoints" = [
        {
          "port"     = "http"
          "interval" = "30s"
        }
      ]
      "jobLabel" = "app.kubernetes.io/name"
      "selector" = {
        "matchLabels" = {
          "app.kubernetes.io/name" = "prometheus"
        }
      }
    }
  }
  
  depends_on = [
    helm_release.kube_prometheus_stack
  ]
}

# Create Grafana dashboards ConfigMap
resource "kubernetes_config_map" "grafana_dashboards" {
  metadata {
    name      = "grafana-dashboards"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
    
    labels = {
      grafana_dashboard = "1"
    }
  }
  
  data = {
    "kubernetes-cluster.json" = file("${path.module}/dashboards/kubernetes-cluster.json")
    "kubernetes-pods.json"   = file("${path.module}/dashboards/kubernetes-pods.json")
    "kubernetes-nodes.json"  = file("${path.module}/dashboards/kubernetes-nodes.json")
  }
  
  depends_on = [
    kubernetes_namespace.monitoring
  ]
}

# Create PrometheusRule for alerting
resource "kubernetes_manifest" "prometheus_rules" {
  manifest = {
    "apiVersion" = "monitoring.coreos.com/v1"
    "kind"       = "PrometheusRule"
    "metadata" = {
      "name"      = "kube-prometheus-stack-alerts"
      "namespace" = kubernetes_namespace.monitoring.metadata[0].name
      "labels" = {
        "app"     = "kube-prometheus-stack"
        "release" = "kube-prometheus-stack"
      }
    }
    "spec" = {
      "groups" = [
        {
          "name" = "kubernetes-apps"
          "rules" = [
            {
              "alert" = "KubePodCrashLooping"
              "annotations" = {
                "description" = "Pod {{ $labels.namespace }}/{{ $labels.pod }} ({{ $labels.container }}) is restarting {{ $value }} times every 5 minutes."
                "summary"     = "Pod is crash looping."
              }
              "expr" = "sum by (namespace, pod, container) (rate(kube_pod_container_status_restarts_total[5m]) * 60 * 5) > 0"
              "for"  = "15m"
              "labels" = {
                "severity" = "warning"
              }
            },
            {
              "alert" = "KubePodNotReady"
              "annotations" = {
                "description" = "Pod {{ $labels.namespace }}/{{ $labels.pod }} has been in a non-ready state for longer than 15 minutes."
                "summary"     = "Pod has been in a non-ready state for more than 15 minutes."
              }
              "expr" = "sum by (namespace, pod) (max by(namespace, pod) (kube_pod_status_phase{phase=~"Pending|Unknown"}) * on(namespace, pod) group_left(owner_kind) max by(namespace, pod, owner_kind) (kube_pod_owner{owner_kind!="Job"})) > 0"
              "for"  = "15m"
              "labels" = {
                "severity" = "warning"
              }
            }
          ]
        }
      ]
    }
  }
  
  depends_on = [
    helm_release.kube_prometheus_stack
  ]
}

# Create Grafana data source for Loki
resource "kubernetes_manifest" "grafana_datasource_loki" {
  count = var.enable_loki ? 1 : 0
  
  manifest = {
    "apiVersion" = "v1"
    "kind"       = "ConfigMap"
    "metadata" = {
      "name"      = "grafana-datasource-loki"
      "namespace" = kubernetes_namespace.monitoring.metadata[0].name
      "labels" = {
        "grafana_datasource" = "1"
      }
    }
    "data" = {
      "loki.yaml" = <<-EOT
        apiVersion: 1
        datasources:
          - name: Loki
            type: loki
            access: proxy
            url: http://loki:3100
            isDefault: false
            version: 1
            editable: true
      EOT
    }
  }
  
  depends_on = [
    helm_release.kube_prometheus_stack,
    helm_release.loki
  ]
}

# Create Grafana dashboard for Loki
resource "kubernetes_config_map" "loki_dashboard" {
  count = var.enable_loki ? 1 : 0
  
  metadata {
    name      = "loki-dashboard"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
    
    labels = {
      grafana_dashboard = "1"
    }
  }
  
  data = {
    "loki.json" = file("${path.module}/dashboards/loki.json")
  }
  
  depends_on = [
    kubernetes_namespace.monitoring
  ]
}
