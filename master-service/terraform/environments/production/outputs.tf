# EKS Cluster Outputs
output "cluster_name" {
  description = "The name of the EKS cluster"
  value       = module.eks.cluster_name
}

output "cluster_endpoint" {
  description = "The endpoint for the EKS cluster"
  value       = module.eks.cluster_endpoint
}

output "cluster_certificate_authority_data" {
  description = "The base64 encoded certificate data required to communicate with the cluster"
  value       = module.eks.cluster_certificate_authority_data
}

output "cluster_oidc_issuer_url" {
  description = "The URL of the OIDC issuer"
  value       = module.eks.cluster_oidc_issuer_url
}

# VPC Outputs
output "vpc_id" {
  description = "The ID of the VPC"
  value       = module.vpc.vpc_id
}

output "public_subnets" {
  description = "List of public subnet IDs"
  value       = module.vpc.public_subnets
}

output "private_subnets" {
  description = "List of private subnet IDs"
  value       = module.vpc.private_subnets
}

# IAM Outputs
output "cluster_role_arn" {
  description = "The ARN of the EKS cluster IAM role"
  value       = module.iam.cluster_role_arn
}

output "node_group_role_arn" {
  description = "The ARN of the EKS node group IAM role"
  value       = module.iam.node_group_role_arn
}

output "cluster_autoscaler_role_arn" {
  description = "The ARN of the Cluster Autoscaler IAM role"
  value       = module.cluster_autoscaler_irsa.iam_role_arn
}

output "load_balancer_controller_role_arn" {
  description = "The ARN of the AWS Load Balancer Controller IAM role"
  value       = module.load_balancer_controller_irsa.iam_role_arn
}

external_dns_role_arn = module.external_dns_irsa.iam_role_arn

# Kubernetes Configuration
output "kubeconfig" {
  description = "kubectl config file contents for this EKS cluster"
  value       = module.eks.kubeconfig
  sensitive   = true
}

output "config_map_aws_auth" {
  description = "A kubernetes configuration to authenticate to this EKS cluster"
  value       = module.eks.config_map_aws_auth
  sensitive   = true
}

# Monitoring Outputs
output "monitoring_namespace" {
  description = "The name of the monitoring namespace"
  value       = module.monitoring.namespace
}

output "prometheus_service_name" {
  description = "The name of the Prometheus service"
  value       = module.monitoring.prometheus_service_name
}

output "prometheus_service_port" {
  description = "The port of the Prometheus service"
  value       = module.monitoring.prometheus_service_port
}

output "alertmanager_service_name" {
  description = "The name of the Alertmanager service"
  value       = module.monitoring.alertmanager_service_name
}

output "alertmanager_service_port" {
  description = "The port of the Alertmanager service"
  value       = module.monitoring.alertmanager_service_port
}

output "grafana_service_name" {
  description = "The name of the Grafana service"
  value       = module.monitoring.grafana_service_name
}

output "grafana_service_port" {
  description = "The port of the Grafana service"
  value       = module.monitoring.grafana_service_port
}

output "loki_service_name" {
  description = "The name of the Loki service"
  value       = module.monitoring.loki_service_name
}

output "loki_service_port" {
  description = "The port of the Loki service"
  value       = module.monitoring.loki_service_port
}

output "prometheus_ingress_host" {
  description = "The hostname for the Prometheus ingress"
  value       = module.monitoring.prometheus_ingress_host
}

output "grafana_admin_password" {
  description = "The admin password for Grafana"
  value       = var.grafana_admin_password != "" ? var.grafana_admin_password : module.monitoring.grafana_admin_password
  sensitive   = true
}

output "monitoring_auth_username" {
  description = "The username for monitoring components basic authentication"
  value       = module.monitoring.basic_auth_username
  sensitive   = true
}

output "monitoring_auth_password" {
  description = "The password for monitoring components basic authentication"
  value       = var.monitoring_auth_password != "" ? var.monitoring_auth_password : module.monitoring.basic_auth_password
  sensitive   = true
}

output "alertmanager_ingress_host" {
  description = "The hostname for the Alertmanager ingress"
  value       = module.monitoring.alertmanager_ingress_host
}

output "loki_ingress_host" {
  description = "The hostname for the Loki ingress"
  value       = module.monitoring.loki_ingress_host
}

# Command to access Prometheus
output "prometheus_access_command" {
  description = "Command to port-forward Prometheus to localhost"
  value       = "kubectl port-forward -n ${module.monitoring.namespace} svc/${module.monitoring.prometheus_service_name} 9090:${module.monitoring.prometheus_service_port}"
}

# Command to access Grafana
output "grafana_access_command" {
  description = "Command to port-forward Grafana to localhost"
  value       = "kubectl port-forward -n ${module.monitoring.namespace} svc/${module.monitoring.grafana_service_name} 3000:${module.monitoring.grafana_service_port}"
}

# Command to access Alertmanager
output "alertmanager_access_command" {
  description = "Command to port-forward Alertmanager to localhost"
  value       = "kubectl port-forward -n ${module.monitoring.namespace} svc/${module.monitoring.alertmanager_service_name} 9093:${module.monitoring.alertmanager_service_port}"
}

# Command to access Loki (if enabled)
output "loki_access_command" {
  description = "Command to port-forward Loki to localhost"
  value       = var.enable_loki ? "kubectl port-forward -n ${module.monitoring.namespace} svc/${module.monitoring.loki_service_name} 3100:${module.monitoring.loki_service_port}" : "Loki is disabled"
}

# Command to get Grafana admin password
output "grafana_admin_password_command" {
  description = "Command to get the Grafana admin password"
  value       = "kubectl get secret --namespace ${module.monitoring.namespace} ${module.monitoring.grafana_service_name} -o jsonpath='{.data.admin-password}' | base64 --decode ; echo"
  sensitive   = true
}

# Command to update kubeconfig
output "update_kubeconfig_command" {
  description = "Command to update kubeconfig for the EKS cluster"
  value       = "aws eks --region ${var.aws_region} update-kubeconfig --name ${module.eks.cluster_name}"
}
