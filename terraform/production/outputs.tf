output "gke_cluster_name" {
  description = "The name of the GKE cluster"
  value       = google_container_cluster.primary.name
}

output "gke_cluster_endpoint" {
  description = "The endpoint for the GKE cluster"
  value       = google_container_cluster.primary.endpoint
}

output "gke_cluster_ca_certificate" {
  description = "The CA certificate for the GKE cluster"
  value       = google_container_cluster.primary.master_auth[0].cluster_ca_certificate
  sensitive   = true
}

output "database_connection_name" {
  description = "The connection name of the Cloud SQL instance"
  value       = google_sql_database_instance.postgres.connection_name
}

output "database_public_ip" {
  description = "The public IP of the database"
  value       = google_sql_database_instance.postgres.public_ip_address
}

output "redis_host" {
  description = "The host of the Redis instance"
  value       = google_redis_instance.cache.host
}

output "redis_port" {
  description = "The port of the Redis instance"
  value       = google_redis_instance.cache.port
}

output "storage_bucket_name" {
  description = "The name of the storage bucket"
  value       = google_storage_bucket.models.name
}

output "vpc_name" {
  description = "The name of the VPC"
  value       = google_compute_network.vpc.name
}

output "subnet_name" {
  description = "The name of the subnet"
  value       = google_compute_subnetwork.subnet.name
}

output "workload_identity_pool" {
  description = "The workload identity pool"
  value       = "${var.project_id}.svc.id.goog"
}

output "node_pool_names" {
  description = "The names of the node pools"
  value = [
    google_container_node_pool.cpu_nodes.name,
    google_container_node_pool.gpu_nodes.name
  ]
}

output "kubernetes_namespace" {
  description = "The Kubernetes namespace"
  value       = var.namespace
}

output "load_balancer_ip" {
  description = "The IP address of the load balancer"
  value       = kubernetes_service.zeta_ingress.load_balancer_ingress[0].ip
}

output "service_account_email" {
  description = "The email of the service account"
  value       = google_service_account.zeta_service_account.email
}
