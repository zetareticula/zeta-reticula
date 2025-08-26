# Production Infrastructure as Code for Zeta Reticula

provider "google" {
  project = var.project_id
  region  = var.region
}

# Enable required Google Cloud APIs
resource "google_project_service" "gcp_services" {
  for_each = toset([
    "compute.googleapis.com",
    "container.googleapis.com",
    "redis.googleapis.com",
    "sqladmin.googleapis.com",
    "monitoring.googleapis.com",
    "logging.googleapis.com",
    "cloudtrace.googleapis.com",
    "profiler.googleapis.com"
  ])
  
  service            = each.key
  disable_on_destroy = false
}

# VPC Network
resource "google_compute_network" "vpc" {
  name                    = "zeta-vpc"
  auto_create_subnetworks = false
  mtu                     = 1460
}

# Subnet
resource "google_compute_subnetwork" "subnet" {
  name          = "zeta-subnet"
  ip_cidr_range = "10.0.0.0/20"
  region        = var.region
  network       = google_compute_network.vpc.id
  
  secondary_ip_range {
    range_name    = "pods"
    ip_cidr_range = "10.1.0.0/16"
  }
  
  secondary_ip_range {
    range_name    = "services"
    ip_cidr_range = "10.2.0.0/20"
  }
}

# GKE Cluster
resource "google_container_cluster" "primary" {
  name     = "zeta-cluster"
  location = var.region
  
  remove_default_node_pool = true
  initial_node_count       = 1
  network                 = google_compute_network.vpc.self_link
  subnetwork              = google_compute_subnetwork.subnet.self_link
  
  networking_mode = "VPC_NATIVE"
  ip_allocation_policy {
    cluster_secondary_range_name  = "pods"
    services_secondary_range_name = "services"
  }
  
  # Enable Workload Identity
  workload_identity_config {
    workload_pool = "${var.project_id}.svc.id.goog"
  }
  
  # Enable private nodes
  private_cluster_config {
    enable_private_nodes    = true
    enable_private_endpoint = false
    master_ipv4_cidr_block  = "172.16.0.0/28"
  }
  
  # Enable network policy
  network_policy {
    enabled = true
  }
  
  # Enable vertical pod autoscaling
  vertical_pod_autoscaling {
    enabled = true
  }
  
  # Enable release channel
  release_channel {
    channel = "STABLE"
  }
  
  # Enable maintenance windows
  maintenance_policy {
    recurring_window {
      start_time = "2023-01-01T00:00:00Z"
      end_time   = "2023-01-01T03:00:00Z"
      recurrence = "FREQ=WEEKLY;BYDAY=SA"
    }
  }
  
  # Enable logging and monitoring
  logging_service    = "logging.googleapis.com/kubernetes"
  monitoring_service = "monitoring.googleapis.com/kubernetes"
  
  # Enable cost allocation
  resource_usage_export_config {
    enable_network_egress_metering = true
    enable_resource_consumption_metering = true
    
    bigquery_destination {
      dataset_id = "zeta_usage_metering"
    }
  }
}

# Node Pool for CPU workloads
resource "google_container_node_pool" "cpu_nodes" {
  name       = "cpu-node-pool"
  location   = var.region
  cluster    = google_container_cluster.primary.name
  node_count = 3
  
  management {
    auto_repair  = true
    auto_upgrade = true
  }
  
  node_config {
    preemptible  = false
    machine_type = "n2-standard-8"
    disk_size_gb = 100
    disk_type    = "pd-ssd"
    
    # Enable workload identity
    workload_metadata_config {
      mode = "GKE_METADATA"
    }
    
    # Taints and tolerations
    taint {
      key    = "dedicated"
      value  = "cpu"
      effect = "NO_SCHEDULE"
    }
    
    # Resource limits
    resource_limits {
      resource_type = "cpu"
      maximum      = 8
    }
    resource_limits {
      resource_type = "memory"
      maximum      = 32768
    }
    
    # OAuth scopes
    oauth_scopes = [
      "https://www.googleapis.com/auth/cloud-platform"
    ]
  }
  
  # Autoscaling configuration
  autoscaling {
    min_node_count = 3
    max_node_count = 10
  }
  
  # Upgrade settings
  upgrade_settings {
    max_surge       = 1
    max_unavailable = 0
  }
}

# Node Pool for GPU workloads
resource "google_container_node_pool" "gpu_nodes" {
  name       = "gpu-node-pool"
  location   = var.region
  cluster    = google_container_cluster.primary.name
  node_count = 1
  
  management {
    auto_repair  = true
    auto_upgrade = true
  }
  
  node_config {
    preemptible  = false
    machine_type = "n1-standard-8"
    disk_size_gb = 200
    disk_type    = "pd-ssd"
    
    # GPU configuration
    guest_accelerator {
      type  = "nvidia-tesla-t4"
      count = 1
    }
    
    # Enable workload identity
    workload_metadata_config {
      mode = "GKE_METADATA"
    }
    
    # Taints and tolerations
    taint {
      key    = "nvidia.com/gpu"
      value  = "present"
      effect = "NO_SCHEDULE"
    }
    
    # Resource limits
    resource_limits {
      resource_type = "nvidia.com/gpu"
      maximum      = 1
    }
    
    # OAuth scopes
    oauth_scopes = [
      "https://www.googleapis.com/auth/cloud-platform"
    ]
  }
  
  # Autoscaling configuration
  autoscaling {
    min_node_count = 1
    max_node_count = 5
  }
  
  # Upgrade settings
  upgrade_settings {
    max_surge       = 1
    max_unavailable = 0
  }
}

# Cloud SQL for PostgreSQL
resource "google_sql_database_instance" "postgres" {
  name             = "zeta-postgres"
  database_version = "POSTGRES_13"
  region           = var.region
  
  settings {
    tier              = "db-custom-4-16384"
    availability_type = "REGIONAL"
    disk_size         = 100
    disk_type         = "PD_SSD"
    
    backup_configuration {
      enabled    = true
      start_time = "02:00"
      location   = var.region
    }
    
    maintenance_window {
      day          = 7
      hour         = 2
      update_track = "stable"
    }
    
    ip_configuration {
      ipv4_enabled    = false
      private_network = google_compute_network.vpc.self_link
    }
  }
  
  deletion_protection = false
}

# Database
resource "google_sql_database" "database" {
  name     = "zetadb"
  instance = google_sql_database_instance.postgres.name
}

# Database user
resource "google_sql_user" "users" {
  name     = "zeta"
  instance = google_sql_database_instance.postgres.name
  password = var.db_password
}

# Memorystore (Redis)
resource "google_redis_instance" "cache" {
  name           = "zeta-redis"
  tier           = "STANDARD_HA"
  memory_size_gb = 10
  region         = var.region
  
  authorized_network = google_compute_network.vpc.self_link
  connect_mode       = "PRIVATE_SERVICE_ACCESS"
  
  redis_version = "REDIS_6_X"
  redis_configs = {
    maxmemory-policy = "allkeys-lru"
  }
  
  maintenance_policy {
    weekly_maintenance_window {
      day = "SUNDAY"
      start_time {
        hours   = 2
        minutes = 0
      }
    }
  }
}

# Cloud Storage for models and data
resource "google_storage_bucket" "models" {
  name          = "${var.project_id}-models"
  location      = var.region
  storage_class = "STANDARD"
  
  versioning {
    enabled = true
  }
  
  lifecycle_rule {
    action {
      type = "Delete"
    }
    condition {
      age = 30
    }
  }
  
  lifecycle_rule {
    action {
      type = "SetStorageClass"
      storage_class = "NEARLINE"
    }
    condition {
      age = 90
    }
  }
}

# IAM bindings
resource "google_project_iam_member" "workload_identity_user" {
  project = var.project_id
  role    = "roles/iam.workloadIdentityUser"
  member  = "serviceAccount:${var.project_id}.svc.id.goog[${var.namespace}/zeta-sa]"
}

# Outputs
output "cluster_name" {
  value = google_container_cluster.primary.name
}

output "cluster_endpoint" {
  value = google_container_cluster.primary.endpoint
}

output "database_connection_name" {
  value = google_sql_database_instance.postgres.connection_name
}

output "redis_host" {
  value = google_redis_instance.cache.host
}
