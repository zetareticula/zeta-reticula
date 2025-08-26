output "namespace" {
  description = "The name of the monitoring namespace"
  value       = kubernetes_namespace.monitoring.metadata[0].name
}

output "prometheus_service_name" {
  description = "The name of the Prometheus service"
  value       = "prometheus-operated"
}

output "prometheus_service_port" {
  description = "The port of the Prometheus service"
  value       = 9090
}

output "alertmanager_service_name" {
  description = "The name of the Alertmanager service"
  value       = "alertmanager-operated"
}

output "alertmanager_service_port" {
  description = "The port of the Alertmanager service"
  value       = 9093
}

output "grafana_service_name" {
  description = "The name of the Grafana service"
  value       = "kube-prometheus-stack-grafana"
}

output "grafana_service_port" {
  description = "The port of the Grafana service"
  value       = 80
}

output "loki_service_name" {
  description = "The name of the Loki service"
  value       = var.enable_loki ? "loki" : ""
}

output "loki_service_port" {
  description = "The port of the Loki service"
  value       = 3100
}

output "promtail_service_account_name" {
  description = "The name of the Promtail service account"
  value       = var.enable_loki ? kubernetes_service_account.promtail[0].metadata[0].name : ""
}

output "promtail_service_account_namespace" {
  description = "The namespace of the Promtail service account"
  value       = var.enable_loki ? kubernetes_namespace.monitoring.metadata[0].name : ""
}

output "prometheus_ingress_host" {
  description = "The hostname for the Prometheus ingress"
  value       = "prometheus.${var.environment}.${var.cluster_name}"
}

output "grafana_ingress_host" {
  description = "The hostname for the Grafana ingress"
  value       = "grafana.${var.environment}.${var.cluster_name}"
}

output "alertmanager_ingress_host" {
  description = "The hostname for the Alertmanager ingress"
  value       = "alertmanager.${var.environment}.${var.cluster_name}"
}

output "loki_ingress_host" {
  description = "The hostname for the Loki ingress (if enabled)"
  value       = var.enable_loki ? "loki.${var.environment}.${var.cluster_name}" : ""
}

output "basic_auth_username" {
  description = "The username for basic authentication"
  value       = var.basic_auth_enabled ? var.basic_auth_username : ""
  sensitive   = true
}

output "basic_auth_password" {
  description = "The password for basic authentication (auto-generated if not provided)"
  value       = var.basic_auth_enabled ? (var.basic_auth_password != "" ? var.basic_auth_password : (length(random_password.basic_auth) > 0 ? random_password.basic_auth[0].result : "")) : ""
  sensitive   = true
}
