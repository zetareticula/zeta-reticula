# General
variable "environment" {
  description = "The environment name (e.g., staging, production)"
  type        = string
  default     = "production"
}

variable "cluster_name" {
  description = "The name of the EKS cluster"
  type        = string
  default     = "zeta-reticula"
}

variable "aws_region" {
  description = "The AWS region to deploy to"
  type        = string
  default     = "us-west-2"
}

# VPC Configuration
variable "vpc_cidr" {
  description = "The CIDR block for the VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "azs" {
  description = "A list of availability zones in the region"
  type        = list(string)
  default     = ["us-west-2a", "us-west-2b", "us-west-2c"]
}

variable "public_subnets" {
  description = "A list of public subnets inside the VPC"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
}

variable "private_subnets" {
  description = "A list of private subnets inside the VPC"
  type        = list(string)
  default     = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]
}

# EKS Configuration
variable "kubernetes_version" {
  description = "Kubernetes version to use for the EKS cluster"
  type        = string
  default     = "1.24"
}

# Node Group Configuration
variable "desired_capacity" {
  description = "Desired number of worker nodes"
  type        = number
  default     = 3
}

variable "max_capacity" {
  description = "Maximum number of worker nodes"
  type        = number
  default     = 10
}

variable "min_capacity" {
  description = "Minimum number of worker nodes"
  type        = number
  default     = 3
}

variable "instance_types" {
  description = "List of instance types associated with the EKS Node Group"
  type        = list(string)
  default     = ["m5.large", "m5a.large"]
}

variable "capacity_type" {
  description = "Type of capacity associated with the EKS Node Group. Valid values: ON_DEMAND, SPOT"
  type        = string
  default     = "ON_DEMAND"
}

variable "disk_size" {
  description = "Disk size in GiB for worker nodes"
  type        = number
  default     = 50
}

# IAM Configuration
variable "map_roles" {
  description = "Additional IAM roles to add to the aws-auth configmap"
  type        = list(any)
  default     = []
}

variable "map_users" {
  description = "Additional IAM users to add to the aws-auth configmap"
  type        = list(any)
  default     = []
}

# Let's Encrypt Configuration
variable "letsencrypt_email" {
  description = "Email address for Let's Encrypt account"
  type        = string
  default     = "admin@example.com"  # Change this to your email
}

# Monitoring Configuration
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
  sensitive   = true
  default     = "" # If empty, a random password will be generated
}

variable "monitoring_auth_username" {
  description = "Username for monitoring components basic authentication"
  type        = string
  default     = "admin"
  sensitive   = true
}

variable "monitoring_auth_password" {
  description = "Password for monitoring components basic authentication. If empty, a random password will be generated."
  type        = string
  sensitive   = true
  default     = ""
}

# Tags
variable "tags" {
  description = "A map of tags to add to all resources"
  type        = map(string)
  default     = {}
}

# Local variables
locals {
  name = "${var.cluster_name}-${var.environment}"
  
  tags = merge(
    {
      "Environment" = var.environment
      "Terraform"   = "true"
      "Project"     = "zeta-reticula"
      "Component"   = "master-service"
    },
    var.tags
  )
}

# Data sources
data "aws_caller_identity" "current" {}

data "aws_availability_zones" "available" {
  state = "available"
}
