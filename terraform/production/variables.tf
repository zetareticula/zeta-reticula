variable "project_id" {
  description = "The GCP project ID"
  type        = string
}

variable "region" {
  description = "The GCP region to deploy to"
  type        = string
  default     = "us-central1"
}

variable "namespace" {
  description = "Kubernetes namespace"
  type        = string
  default     = "zeta-prod"
}

variable "db_password" {
  description = "Database password"
  type        = string
  sensitive   = true
}

variable "redis_password" {
  description = "Redis password"
  type        = string
  sensitive   = true
}

variable "environment" {
  description = "Environment name (e.g., prod, staging)"
  type        = string
  default     = "prod"
}

variable "node_locations" {
  description = "List of zones for the cluster"
  type        = list(string)
  default     = ["us-central1-a", "us-central1-b", "us-central1-f"]
}

variable "min_node_count" {
  description = "Minimum number of nodes in the node pool"
  type        = number
  default     = 3
}

variable "max_node_count" {
  description = "Maximum number of nodes in the node pool"
  type        = number
  default     = 10
}

variable "machine_type" {
  description = "Machine type for the nodes"
  type        = string
  default     = "n2-standard-4"
}

variable "disk_size_gb" {
  description = "Disk size in GB for the nodes"
  type        = number
  default     = 100
}

variable "preemptible_nodes" {
  description = "Use preemptible nodes"
  type        = bool
  default     = false
}

variable "enable_private_nodes" {
  description = "Enable private nodes for the cluster"
  type        = bool
  default     = true
}

variable "master_ipv4_cidr_block" {
  description = "CIDR block for the master's VPC"
  type        = string
  default     = "172.16.0.0/28"
}

variable "pods_ipv4_cidr_block" {
  description = "CIDR block for pods"
  type        = string
  default     = "10.1.0.0/16"
}

variable "services_ipv4_cidr_block" {
  description = "CIDR block for services"
  type        = string
  default     = "10.2.0.0/20"
}

variable "database_tier" {
  description = "Database instance tier"
  type        = string
  default     = "db-custom-4-16384"
}

variable "database_disk_size" {
  description = "Database disk size in GB"
  type        = number
  default     = 100
}

variable "redis_memory_size_gb" {
  description = "Redis memory size in GB"
  type        = number
  default     = 10
}

variable "enable_workload_identity" {
  description = "Enable Workload Identity"
  type        = bool
  default     = true
}

variable "enable_vertical_pod_autoscaling" {
  description = "Enable Vertical Pod Autoscaling"
  type        = bool
  default     = true
}

variable "enable_horizontal_pod_autoscaling" {
  description = "Enable Horizontal Pod Autoscaling"
  type        = bool
  default     = true
}

variable "enable_network_policy" {
  description = "Enable network policy"
  type        = bool
  default     = true
}

variable "enable_http_load_balancing" {
  description = "Enable HTTP load balancing"
  type        = bool
  default     = true
}

variable "enable_cloudrun" {
  description = "Enable Cloud Run"
  type        = bool
  default     = false
}

variable "enable_istio" {
  description = "Enable Istio"
  type        = bool
  default     = true
}

variable "istio_auth" {
  description = "Enable Istio mTLS"
  type        = string
  default     = "AUTH_MUTUAL_TLS"
}
