variable "environment" {
  description = "The environment name (e.g., staging, production)"
  type        = string
}

variable "cluster_name" {
  description = "The name of the EKS cluster"
  type        = string
  default     = "zeta-reticula"
}

variable "enable_loki" {
  description = "Enable Loki for log aggregation"
  type        = bool
  default     = true
}

variable "prometheus_storage_class" {
  description = "Storage class for Prometheus persistent volume"
  type        = string
  default     = "gp2"
}

variable "prometheus_storage_size" {
  description = "Storage size for Prometheus persistent volume"
  type        = string
  default     = "50Gi"
}

variable "prometheus_retention" {
  description = "Retention period for Prometheus metrics"
  type        = string
  default     = "15d"
}

variable "loki_storage_class" {
  description = "Storage class for Loki persistent volume"
  type        = string
  default     = "gp2"
}

variable "loki_storage_size" {
  description = "Storage size for Loki persistent volume"
  type        = string
  default     = "100Gi"
}

variable "alertmanager_enabled" {
  description = "Enable Alertmanager"
  type        = bool
  default     = true
}

variable "grafana_admin_password" {
  description = "Admin password for Grafana"
  type        = string
  default     = "admin"
  sensitive   = true
}

variable "ingress_enabled" {
  description = "Enable Ingress resources for monitoring components"
  type        = bool
  default     = true
}

variable "ingress_class_name" {
  description = "Name of the IngressClass to use for the Ingress resources"
  type        = string
  default     = "nginx"
}

variable "tls_enabled" {
  description = "Enable TLS for Ingress resources"
  type        = bool
  default     = true
}

variable "basic_auth_enabled" {
  description = "Enable basic authentication for sensitive endpoints (Prometheus, Alertmanager)"
  type        = bool
  default     = true
}

variable "basic_auth_username" {
  description = "Username for basic authentication"
  type        = string
  default     = "admin"
  sensitive   = true
}

variable "basic_auth_password" {
  description = "Password for basic authentication"
  type        = string
  sensitive   = true
  default     = "" # If empty, a random password will be generated
}

variable "tags" {
  description = "A map of tags to add to all resources"
  type        = map(string)
  default     = {}
}
